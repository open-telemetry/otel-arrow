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

package experimentprocessor // import "github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor"

import (
	"errors"
	"fmt"
)

var (
	errNoExporters     = errors.New("no exporters defined for the route")
	errNoTableItems    = errors.New("the routing table is empty")
	errZeroTableWeight = errors.New("zero weight table")
	errInvalidWeight   = errors.New("negative weight is invalid")
)

// Config defines configuration for the Routing processor.
type Config struct {
	// Table contains the routing table for this processor.
	// Required, must be non-empty.
	Table []RoutingTableItem `mapstructure:"table"`
}

// Validate checks if the processor configuration is valid.
func (c *Config) Validate() error {
	// validate that there's at least one item in the table
	if len(c.Table) == 0 {
		return fmt.Errorf("invalid route table: %w", errNoTableItems)
	}

	// validate each route table item
	total := 0
	for _, item := range c.Table {
		if item.Weight < 0 {
			return fmt.Errorf("invalid route weight %d: %w", item.Weight, errInvalidWeight)
		}

		if len(item.Exporters) == 0 {
			return fmt.Errorf("invalid route entry: %w", errNoExporters)
		}
		total += item.Weight
	}
	if total == 0 {
		return fmt.Errorf("invalid route table: %w", errZeroTableWeight)
	}

	return nil
}

// RoutingTableItem specifies how data should be routed to the different exporters
type RoutingTableItem struct {
	// Weight is relative weight within the table.
	Weight int `mapstructure:"weight"`

	// Exporters contains the list of exporters to use when this
	// table item is selected.  Must be non-empty.
	Exporters []string `mapstructure:"exporters"`
}
