# Admin Types

`otap-df-admin-types` contains the shared request, response, query, and model
types used by the OTAP Dataflow admin server and the public admin SDK.

This crate is an internal workspace boundary. External integrators should
depend on `otap-df-admin-api`, which re-exports these shared model modules as
part of the public SDK surface.
