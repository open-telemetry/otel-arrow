# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

FROM postgres:15.19

COPY init.sql /docker-entrypoint-initdb.d/10-receiver-fixture.sql
COPY start-postgres.sh /usr/local/bin/start-postgres-tls

RUN sed -i 's/\r$//' /usr/local/bin/start-postgres-tls \
    /docker-entrypoint-initdb.d/10-receiver-fixture.sql \
    && chmod 0755 /usr/local/bin/start-postgres-tls

ENTRYPOINT ["/usr/local/bin/start-postgres-tls"]
