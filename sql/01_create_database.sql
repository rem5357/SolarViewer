-- Create StellarForge database with PostGIS support
-- Run this as a superuser to create the initial database

-- Drop and recreate database (WARNING: This will delete all data!)
-- Comment out these lines after initial setup
-- DROP DATABASE IF EXISTS stellarforge;
-- CREATE DATABASE stellarforge;

-- Connect to stellarforge database
\c stellarforge;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";      -- For UUID generation
CREATE EXTENSION IF NOT EXISTS postgis;          -- Spatial support
CREATE EXTENSION IF NOT EXISTS postgis_topology; -- Topology support
CREATE EXTENSION IF NOT EXISTS pg_trgm;          -- For fuzzy text search
CREATE EXTENSION IF NOT EXISTS btree_gist;       -- For exclusion constraints

-- Create schema for better organization
CREATE SCHEMA IF NOT EXISTS stellar;
CREATE SCHEMA IF NOT EXISTS political;
CREATE SCHEMA IF NOT EXISTS routing;

-- Set search path
SET search_path TO stellar, political, routing, public;

-- Create custom types
CREATE TYPE stellar.body_kind AS ENUM (
    'star',
    'planet',
    'moon',
    'station',
    'asteroid_belt',
    'asteroid',
    'comet',
    'planetoid',
    'wreck',
    'artifact',
    'nebula',
    'lagrange_point',
    'vehicle',
    'rogue_planet',
    'binary_system',
    'custom'
);

CREATE TYPE stellar.system_type AS ENUM (
    'single',
    'binary',
    'multiple',
    'cluster',
    'nebula'
);

CREATE TYPE stellar.spectral_class AS ENUM (
    'O', 'B', 'A', 'F', 'G', 'K', 'M',  -- Main sequence
    'L', 'T', 'Y',  -- Brown dwarfs
    'C', 'S',       -- Carbon stars
    'W',            -- Wolf-Rayet
    'Unknown'
);

CREATE TYPE stellar.coordinate_system AS ENUM (
    'galactic_iau',    -- Standard IAU galactic
    'icrs',            -- International Celestial Reference System
    'equatorial_j2000', -- J2000 equatorial
    'astrosynthesis',  -- Legacy Astrosynthesis
    'custom'
);

CREATE TYPE political.government_type AS ENUM (
    'democracy',
    'republic',
    'monarchy',
    'oligarchy',
    'dictatorship',
    'theocracy',
    'anarchy',
    'confederation',
    'federation',
    'empire',
    'corporate',
    'hive_mind',
    'ai_governance',
    'military_junta',
    'tribal',
    'unknown'
);

-- Grant usage on schemas
GRANT USAGE ON SCHEMA stellar TO PUBLIC;
GRANT USAGE ON SCHEMA political TO PUBLIC;
GRANT USAGE ON SCHEMA routing TO PUBLIC;

-- Grant usage on types
GRANT USAGE ON TYPE stellar.body_kind TO PUBLIC;
GRANT USAGE ON TYPE stellar.system_type TO PUBLIC;
GRANT USAGE ON TYPE stellar.spectral_class TO PUBLIC;
GRANT USAGE ON TYPE stellar.coordinate_system TO PUBLIC;
GRANT USAGE ON TYPE political.government_type TO PUBLIC;