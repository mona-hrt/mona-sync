CREATE TABLE IF NOT EXISTS supply_items (
    id TEXT PRIMARY KEY NOT NULL,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    totalDose TEXT,
    usedDose TEXT,
    concentration TEXT,
    moleculeJson TEXT,
    administrationRouteName TEXT,
    esterName TEXT,
    amount INTEGER,
    updatedAt INTEGER NOT NULL,
    isDeleted BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS medication_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    dose TEXT NOT NULL,
    intervalDays INTEGER NOT NULL,
    startDate TEXT NOT NULL,
    moleculeJson TEXT NOT NULL,
    administrationRouteName TEXT NOT NULL,
    esterName TEXT,
    notificationTimes TEXT NOT NULL,
    updatedAt INTEGER NOT NULL,
    isDeleted BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS medication_intakes (
    id TEXT PRIMARY KEY NOT NULL,
    scheduledDateTime TEXT NOT NULL,
    takenDateTime TEXT,
    takenTimeZone TEXT,
    dose TEXT NOT NULL,
    scheduleId TEXT,
    side TEXT,
    moleculeJson TEXT NOT NULL,
    administrationRouteName TEXT NOT NULL,
    esterName TEXT,
    supplyItemId TEXT,
    notes TEXT,
    updatedAt INTEGER NOT NULL,
    isDeleted BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (supplyItemId) REFERENCES supply_items(id) ON DELETE SET NULL,
    FOREIGN KEY (scheduleId) REFERENCES medication_schedules(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS blood_tests (
    id TEXT PRIMARY KEY NOT NULL,
    dateTime TEXT NOT NULL,
    timeZone TEXT NOT NULL,
    estradiolLevels TEXT,
    testosteroneLevels TEXT,
    estradiolUnit TEXT,
    testosteroneUnit TEXT,
    updatedAt INTEGER NOT NULL,
    isDeleted BOOLEAN NOT NULL DEFAULT 0
);

-- Indexes for extremely fast sync pulls
CREATE INDEX idx_supply_items_updated ON supply_items(updatedAt);
CREATE INDEX idx_medication_schedules_updated ON medication_schedules(updatedAt);
CREATE INDEX idx_medication_intakes_updated ON medication_intakes(updatedAt);
CREATE INDEX idx_blood_tests_updated ON blood_tests(updatedAt);
