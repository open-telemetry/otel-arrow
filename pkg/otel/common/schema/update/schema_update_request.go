/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package update

// SchemaUpdateRequest is a counter that keeps track of the number of schema
// update requests.
type SchemaUpdateRequest struct {
	count int
}

// NewSchemaUpdateRequest creates a new SchemaUpdateRequest.
func NewSchemaUpdateRequest() *SchemaUpdateRequest {
	return &SchemaUpdateRequest{count: 0}
}

// Inc increments the counter of the schema update request by one.
func (r *SchemaUpdateRequest) Inc() {
	r.count++
}

// Count returns the current count of the schema update request.
func (r *SchemaUpdateRequest) Count() int {
	return r.count
}

// Reset resets the counter of the schema update request to zero.
func (r *SchemaUpdateRequest) Reset() {
	r.count = 0
}
