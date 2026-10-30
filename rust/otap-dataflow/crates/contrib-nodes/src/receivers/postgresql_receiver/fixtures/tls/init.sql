-- Copyright The OpenTelemetry Authors
-- SPDX-License-Identifier: Apache-2.0

CREATE ROLE otel_reader LOGIN PASSWORD 'otel-reader-password';

CREATE TABLE public.receiver_events (
  event_ts timestamptz(6) NOT NULL,
  event_id bigint NOT NULL,
  actor text NOT NULL,
  scalar_ts timestamptz(6) NOT NULL UNIQUE,
  PRIMARY KEY (event_ts, event_id),
  UNIQUE (event_id)
);

INSERT INTO public.receiver_events VALUES
  ('2026-10-05T00:00:01Z', 1, 'alice', '2026-10-05T00:00:01.000001Z'),
  ('2026-10-05T00:00:01Z', 2, 'bob', '2026-10-05T00:00:01.000002Z'),
  ('2026-10-05T00:00:02.123456Z', 3, 'carol', '2026-10-05T00:00:02.123456Z');

GRANT CONNECT ON DATABASE receiver_fixture TO otel_reader;
GRANT USAGE ON SCHEMA public TO otel_reader;
GRANT SELECT ON public.receiver_events TO otel_reader;
