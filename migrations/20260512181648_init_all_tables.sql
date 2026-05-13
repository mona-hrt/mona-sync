CREATE TABLE IF NOT EXISTS supply_items (
    id TEXT PRIMARY KEY NOT NULL,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    total_dose TEXT,
    used_dose TEXT,
    concentration TEXT,
    molecule_json TEXT,
    administration_route_name TEXT,
    ester_name TEXT,
    amount INTEGER,
    updated_at INTEGER NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS medication_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    dose TEXT NOT NULL,
    interval_days INTEGER NOT NULL,
    start_date TEXT NOT NULL,
    molecule_json TEXT NOT NULL,
    administration_route_name TEXT NOT NULL,
    ester_name TEXT,
    notification_times TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS medication_intakes (
    id TEXT PRIMARY KEY NOT NULL,
    scheduled_date_time TEXT NOT NULL,
    taken_date_time TEXT,
    taken_time_zone TEXT,
    dose TEXT NOT NULL,
    schedule_id TEXT,
    side TEXT,
    molecule_json TEXT NOT NULL,
    administration_route_name TEXT NOT NULL,
    ester_name TEXT,
    supply_item_id TEXT,
    notes TEXT,
    updated_at INTEGER NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (supply_item_id) REFERENCES supply_items(id) ON DELETE SET NULL,
    FOREIGN KEY (schedule_id) REFERENCES medication_schedules(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS blood_tests (
    id TEXT PRIMARY KEY NOT NULL,
    date_time TEXT NOT NULL,
    time_zone TEXT NOT NULL,
    estradiol_levels TEXT,
    testosterone_levels TEXT,
    estradiol_unit TEXT,
    testosterone_unit TEXT,
    updated_at INTEGER NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT 0
);

-- Indexes for extremely fast sync pulls
CREATE INDEX idx_supply_items_updated ON supply_items(updated_at);
CREATE INDEX idx_medication_schedules_updated ON medication_schedules(updated_at);
CREATE INDEX idx_medication_intakes_updated ON medication_intakes(updated_at);
CREATE INDEX idx_blood_tests_updated ON blood_tests(updated_at);
