// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package otlp

import (
	"github.com/apache/arrow/go/v9/arrow"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/ptrace"

	"github.com/f5/otel-arrow-adapter/pkg/air"
	common_arrow "github.com/f5/otel-arrow-adapter/pkg/otel/common/arrow"
	"github.com/f5/otel-arrow-adapter/pkg/otel/constants"

	"go.opentelemetry.io/collector/pdata/pcommon"
)

// Entities is a constraint representing the top level OTLP entities supported by the generic OTLP producer.
type Entities interface {
	ptrace.Traces | plog.Logs
}

// Entity is a constraint representing the bottom level OTLP entities supported by the generic OTLP producer.
type Entity interface {
	ptrace.Span | plog.LogRecord
}

// EntitiesProducer is the main interface used to configure the generic OTLP producer.
type EntitiesProducer[ES Entities, E Entity] interface {
	NewTopLevelEntities() TopLevelEntities[ES, E]
	ResourceEntitiesLabel() string
	ScopeEntitiesLabel() string
	EntitiesLabel() string
	EntityProducer(ScopeEntities[E], *air.ListOfStructs, int) error
}

// TopLevelEntities is the interface representing top level OTLP entities.
type TopLevelEntities[ES Entities, E Entity] interface {
	ResourceEntities() ResourceEntitiesSlice[E]
	Unwrap() ES
}

// ResourceEntitiesSlice is the interface representing a slice of OTLP resource entities.
type ResourceEntitiesSlice[E Entity] interface {
	EnsureCapacity(int)
	AppendEmpty() ResourceEntities[E]
}

// ResourceEntities is the interface representing a OTLP resource entities.
type ResourceEntities[E Entity] interface {
	Resource() pcommon.Resource
	SetSchemaUrl(string)
	ScopeEntities() ScopeEntities[E]
}

// ScopeEntities is the interface representing a OTLP scope entities.
type ScopeEntities[E Entity] interface {
	Scope() pcommon.InstrumentationScope
	SetSchemaUrl(string)
	Entity() E
}

// Producer produces OTLP entities from OTLP Arrow traces.
//
// Must use New to create new instances.
type Producer[ES Entities, E Entity] struct {
	entitiesProducer EntitiesProducer[ES, E]
}

// New is a constructor to create new OTLP producers.
func New[ES Entities, E Entity](entitiesProducer EntitiesProducer[ES, E]) *Producer[ES, E] {
	return &Producer[ES, E]{
		entitiesProducer: entitiesProducer,
	}
}

// ProduceFrom produces OTLP entities from an Arrow Record.
func (p *Producer[ES, E]) ProduceFrom(record arrow.Record) ([]ES, error) {
	// Each first level row in the Arrow Record represents
	//a resource entities (e.g. ResourceSpans, ResourceLogs).
	resEntCount := int(record.NumRows())
	allEntities := make([]ES, 0, resEntCount)

	for entityIdx := 0; entityIdx < resEntCount; entityIdx++ {
		entities := p.entitiesProducer.NewTopLevelEntities()

		arrowResEnts, err := air.ListOfStructsFromRecord(record, p.entitiesProducer.ResourceEntitiesLabel(), entityIdx)
		if err != nil {
			return allEntities, err
		}
		resEntities := entities.ResourceEntities()
		resEntities.EnsureCapacity(arrowResEnts.End() - arrowResEnts.Start())

		for resEntIdx := arrowResEnts.Start(); resEntIdx < arrowResEnts.End(); resEntIdx++ {
			resEnt := resEntities.AppendEmpty()

			resource, err := common_arrow.NewResourceFrom(arrowResEnts, resEntIdx)
			if err != nil {
				return allEntities, err
			}
			resource.CopyTo(resEnt.Resource())

			schemaUrl, err := arrowResEnts.StringFieldByName(constants.SCHEMA_URL, resEntIdx)
			if err != nil {
				return allEntities, err
			}
			resEnt.SetSchemaUrl(schemaUrl)

			arrowScopeEntities, err := arrowResEnts.ListOfStructsByName(p.entitiesProducer.ScopeEntitiesLabel(), resEntIdx)
			if err != nil {
				return allEntities, err
			}
			for scopeEntIdx := arrowScopeEntities.Start(); scopeEntIdx < arrowScopeEntities.End(); scopeEntIdx++ {
				scopeEnt := resEnt.ScopeEntities()

				scope, err := common_arrow.NewScopeFrom(arrowScopeEntities, scopeEntIdx)
				if err != nil {
					return allEntities, err
				}
				scope.CopyTo(scopeEnt.Scope())

				schemaUrl, err := arrowScopeEntities.StringFieldByName(constants.SCHEMA_URL, scopeEntIdx)
				if err != nil {
					return allEntities, err
				}
				scopeEnt.SetSchemaUrl(schemaUrl)

				arrowEntities, err := arrowScopeEntities.ListOfStructsByName(p.entitiesProducer.EntitiesLabel(), scopeEntIdx)
				if err != nil {
					return allEntities, err
				}
				for entityIdx := arrowEntities.Start(); entityIdx < arrowEntities.End(); entityIdx++ {
					err = p.entitiesProducer.EntityProducer(scopeEnt, arrowEntities, entityIdx)
					if err != nil {
						return allEntities, err
					}
				}
			}
		}

		allEntities = append(allEntities, entities.Unwrap())
	}

	return allEntities, nil
}
