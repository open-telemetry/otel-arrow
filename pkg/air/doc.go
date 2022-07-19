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

/*
Package air is an Arrow Intermediate Representation used to translate row-oriented data into column-oriented data.

The idea is to represent each row-oriented entity as a record composed of fields (more or less complex). These records
are then injected into the RecordRepository to build one or more batches of data in columnar format. A series of
optimizations is performed during this process to improve the compression rate of the Arrow records, e.g. automatic
creation of Arrow dictionaries for string and binary fields, sorting the records according to one or more columns.
*/
package air
