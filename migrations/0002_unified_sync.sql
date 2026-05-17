-- Drop existing tables
DROP TABLE IF EXISTS supply_items;
DROP TABLE IF EXISTS medication_schedules;
DROP TABLE IF EXISTS medication_intakes;
DROP TABLE IF EXISTS blood_tests;

-- Create unified sync table
CREATE TABLE IF NOT EXISTS sync_items (
    id TEXT PRIMARY KEY NOT NULL,
    collection TEXT NOT NULL,
    payload TEXT NOT NULL,
    updatedAt INTEGER NOT NULL,
    isDeleted BOOLEAN NOT NULL DEFAULT 0
);

-- Indexes for fast sync
CREATE INDEX idx_sync_items_updated ON sync_items(updatedAt);
CREATE INDEX idx_sync_items_collection_updated ON sync_items(collection, updatedAt);
