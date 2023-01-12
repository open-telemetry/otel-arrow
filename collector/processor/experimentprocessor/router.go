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

package experimentprocessor // import "github.com/f5/otel-arrow-adapter/collector/processor/experimentprocessor"

import (
	"errors"
	"fmt"
	"math/rand"

	"go.opentelemetry.io/collector/component"
	"go.uber.org/zap"
)

// This code is derived from collector-contrib/processor/routingprocessor.
// It is substantially simpler because it routes whole data items, not individual
// points, therefore there is no regrouping step.

var errExporterNotFound = errors.New("exporter not found")

// router registers exporters and default exporters for an exporter. router can
// be instantiated with exporter.Traces, exporter.Metrics, and
// exporter.Logs type arguments.
type router[E component.Component] struct {
	logger *zap.Logger

	randIntn func(int) int
	table    []RoutingTableItem
	routes   []routingItem[E]
}

// newRouter creates a new router instance with its type parameter constrained
// to component.Component.
func newRouter[E component.Component](
	table []RoutingTableItem,
	settings component.TelemetrySettings,
) router[E] {
	return router[E]{
		logger:   settings.Logger,
		randIntn: rand.New(rand.NewSource(rand.Int63())).Intn,
		table:    table,
		routes:   make([]routingItem[E], len(table)),
	}
}

type routingItem[E component.Component] struct {
	exporters []E
	cumWeight int
}

func (r *router[E]) registerExporters(available map[component.ID]component.Component) error {
	total := 0
	for idx, item := range r.table {
		route := &r.routes[idx]
		total += item.Weight
		route.cumWeight = total

		for _, name := range item.Exporters {
			e, err := r.extractExporter(name, available)
			if errors.Is(err, errExporterNotFound) {
				continue
			}
			if err != nil {
				return err
			}
			route.exporters = append(route.exporters, e)
		}
	}
	return nil
}

// extractExporter returns an exporter for the given name (type/name) and type
// argument if it exists in the list of available exporters.
func (r *router[E]) extractExporter(name string, available map[component.ID]component.Component) (E, error) {
	var exporter E

	id := component.ID{}
	if err := id.UnmarshalText([]byte(name)); err != nil {
		return exporter, err
	}
	v, ok := available[id]
	if !ok {
		r.logger.Warn(
			"Can't find the exporter for the routing processor for this pipeline type."+
				" This is OK if you did not specify this processor for that pipeline type",
			zap.Any("pipeline_type", new(E)),
			zap.Error(
				fmt.Errorf(
					"error registering exporter %q",
					name,
				),
			),
		)
		return exporter, errExporterNotFound
	}
	exporter, ok = v.(E)
	if !ok {
		return exporter,
			fmt.Errorf("the exporter %q isn't a %T exporter", id.String(), new(E))
	}
	return exporter, nil
}

func (r *router[E]) getExporters() []E {
	n := len(r.routes)

	// Generate a random number in the range [0, totalWeight)
	// using the last item's cumulative weight.
	x := r.randIntn(r.routes[n-1].cumWeight)

	// Note: linear search. We could use sort.Search() but with a
	// small routing table this is likely faster.
	for _, route := range r.routes[:n-1] {
		if route.cumWeight > x {
			return route.exporters
		}
	}
	return r.routes[n-1].exporters
}
