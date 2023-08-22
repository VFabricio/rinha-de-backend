CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE OR REPLACE FUNCTION public.array_to_string_immutable(text[], text)
 RETURNS text
 LANGUAGE internal IMMUTABLE PARALLEL SAFE STRICT AS 'array_to_text';

CREATE TABLE persons (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  nickname text NOT NULL UNIQUE,
  name text NOT NULL,
  birthdate date NOT NULL,
  stack text[] NOT NULL,
  search text GENERATED ALWAYS AS (name || ' ' || nickname || ' ' || array_to_string_immutable(stack, ' ')) STORED
);

CREATE INDEX search_idx ON persons USING gist(search gist_trgm_ops(siglen=256));
