CREATE TABLE link
(
    id                 text PRIMARY KEY,
    url                text  NOT NULL,
    development_fields jsonb NOT NULL DEFAULT '{}'
);
