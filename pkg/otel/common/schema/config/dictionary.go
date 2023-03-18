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

package builder

// Dictionary is a configuration for a dictionary field.
// The MaxCard is the maximum cardinality of the dictionary field. If the
// cardinality of the dictionary field is higher than MaxCard, then the
// dictionary field will be automatically converted to its base type.
//
// if MaxCard is equal to 0, then the dictionary field will be converted to its
// base type no matter what.
type Dictionary struct {
	MaxCard uint64
}
