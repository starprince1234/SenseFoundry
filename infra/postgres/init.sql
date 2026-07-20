CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

\ir /docker-entrypoint-initdb.d/migrations/V001__initial_schema.sql
\ir /docker-entrypoint-initdb.d/migrations/V002__audit_triggers.sql
\ir /docker-entrypoint-initdb.d/migrations/V003__indexes.sql
\ir /docker-entrypoint-initdb.d/migrations/V004__source_verification.sql
\ir /docker-entrypoint-initdb.d/migrations/V005__corpus_ingestion_rejections.sql
\ir /docker-entrypoint-initdb.d/migrations/V006__model_registry_versions.sql
