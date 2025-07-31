CREATE TYPE AGREEMENT_STATUS AS ENUM (
    'NotInitiated',
   'Initiated',
   'Rejected',
   'Generated',
   'HalfSigned',
   'Signed');

-- Agreements
CREATE TABLE agreements
(
    id                 UUID PRIMARY KEY,
    tenant_id          TEXT NOT NULL,
    landlord_id        TEXT NOT NULL,
    housing_id         UUID NOT NULL,

    -- acceptance flags: TRUE = accepted, FALSE = rejected, NULL = not answered yet
    tenant_accepted    BOOLEAN,
    landlord_accepted  BOOLEAN,

    -- contract artefacts
    doc_generated_at   TIMESTAMPTZ,                   -- file exists => “Generated”
    tenant_signature   TEXT,
    landlord_signature TEXT,

    -- 3. Status is GENERATED (never written by application code)
    status             AGREEMENT_STATUS
        GENERATED ALWAYS AS (
            CASE
                WHEN tenant_signature IS NOT NULL     -- both signed
                    AND landlord_signature IS NOT NULL
                    THEN 'Signed'::AGREEMENT_STATUS

                WHEN tenant_signature IS NOT NULL     -- one signature => half‑signed
                    OR landlord_signature IS NOT NULL
                    THEN 'HalfSigned'::AGREEMENT_STATUS

                WHEN doc_generated_at IS NOT NULL     -- PDF exists
                    THEN 'Generated'::AGREEMENT_STATUS

                WHEN tenant_accepted = FALSE          -- someone rejected
                    OR landlord_accepted = FALSE
                    THEN 'Rejected'::AGREEMENT_STATUS

                WHEN tenant_accepted = TRUE           -- someone accepted
                    OR landlord_accepted = TRUE
                    THEN 'Initiated'::AGREEMENT_STATUS

                ELSE 'NotInitiated'::AGREEMENT_STATUS -- default
                END
            ) STORED,                                 -- STORED = materialised, therefore indexable
    FOREIGN KEY (housing_id)
        REFERENCES housings (id)
        ON DELETE CASCADE
);

ALTER TABLE agreements
    -- no signing after a reject
    ADD CONSTRAINT sig_needs_accept
        CHECK (
            (tenant_signature IS NULL OR tenant_accepted IS TRUE)
                AND
            (landlord_signature IS NULL OR landlord_accepted IS TRUE)
            ),

  -- both acceptances before we generate the document
  ADD CONSTRAINT gen_needs_accept
  CHECK (
    doc_generated_at IS NULL
    OR (tenant_accepted IS TRUE AND landlord_accepted IS TRUE)
  );


-- Document Units (passports, taxpayer cards)
CREATE TABLE document_units
(
    user_id           TEXT PRIMARY KEY,     -- unique row identifier
    taxpayer_card     JSONB       NOT NULL, -- taxpayer data in binary‑JSON
    internal_passport JSONB       NOT NULL, -- passport data in binary‑JSON
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE
EXTENSION IF NOT EXISTS pgcrypto;


-- Extra property (price list)
CREATE TABLE extra_housing_items
(
    id         UUID    NOT NULL DEFAULT gen_random_uuid(),
    housing_id UUID    NOT NULL,
    name       TEXT    NOT NULL,
    price      NUMERIC NOT NULL,
    currency   TEXT    NOT NULL,
    number     INTEGER NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (housing_id)
        REFERENCES housings (id)
        ON DELETE CASCADE
);

CREATE TABLE housings
(
    id                   UUID NOT NULL DEFAULT gen_random_uuid(),
    user_id              TEXT NOT NULL,
    title_image          TEXT,
    apartment_number     TEXT,
    area                 NUMERIC,
    city                 TEXT,
    region               TEXT,
    street               TEXT,
    house_number         TEXT,
    floor_number         TEXT,
    living_complex       TEXT,
    house_type           TEXT,
    number_of_rooms      INTEGER,
    description          TEXT,
    price                NUMERIC,
    currency             TEXT,
    created_at           TIMESTAMPTZ,
    has_air_conditioning BOOLEAN,
    has_balcony          BOOLEAN,
    is_children_friendly BOOLEAN,
    has_dishwasher       BOOLEAN,
    has_filtered_water   BOOLEAN,
    is_for_rent          BOOLEAN,
    has_gas_heating      BOOLEAN,
    has_gas_stove        BOOLEAN,
    has_microwave        BOOLEAN,
    has_oven             BOOLEAN,
    has_parking          BOOLEAN,
    is_pet_friendly      BOOLEAN,
    has_refrigerator     BOOLEAN,
    has_tv               BOOLEAN,
    has_washing_machine  BOOLEAN,
    has_wifi             BOOLEAN,
    PRIMARY KEY (id)
);

CREATE TABLE public.signatures
(
    tenant_id          TEXT NOT NULL,
    landlord_id        TEXT NOT NULL,
    housing_id         UUID NOT NULL,
    tenant_signature   TEXT,
    landlord_signature TEXT,
    PRIMARY KEY (tenant_id, landlord_id, housing_id)
);