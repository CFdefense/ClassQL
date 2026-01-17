CREATE TABLE schools (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE term_collections (
    id TEXT,
    school_id TEXT,

    year INTEGER NOT NULL,
    season TEXT NOT NULL CHECK( season IN ('Spring', 'Fall', 'Winter', 'Summer') ),
    name TEXT,
    still_collecting INTEGER NOT NULL CHECK(still_collecting IN (0, 1)),
    FOREIGN KEY (school_id) REFERENCES schools(id) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (id, school_id)
);

CREATE TABLE professors (
    id TEXT,
    school_id TEXT,

    name TEXT NOT NULL,
    email_address TEXT,
    first_name TEXT,
    last_name TEXT,
    other TEXT,
    FOREIGN KEY (school_id) REFERENCES schools(id) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (id, school_id),
    CHECK(other IS NULL OR json_valid(other))
);


CREATE TABLE courses (
    school_id TEXT,
    subject_code TEXT,
    number TEXT,

    subject_description TEXT,
    title TEXT,
    description TEXT,
    credit_hours REAL NOT NULL,
    prerequisites TEXT,
    corequisites TEXT,
    other TEXT,

    FOREIGN KEY (school_id) REFERENCES schools(id) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (school_id, subject_code, number),
    CHECK(other IS NULL OR json_valid(other))
);


CREATE TABLE sections (
    sequence TEXT,
    term_collection_id TEXT,
    subject_code TEXT,
    course_number TEXT,
    school_id TEXT,

    max_enrollment INTEGER,
    instruction_method TEXT,
    campus TEXT,
    enrollment INTEGER,
    primary_professor_id TEXT,
    other TEXT,
    FOREIGN KEY (school_id, subject_code, course_number)
        REFERENCES courses(school_id, subject_code, number) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (primary_professor_id, school_id) REFERENCES professors(id, school_id) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (term_collection_id, school_id) REFERENCES term_collections(id, school_id) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (sequence, term_collection_id, subject_code, course_number, school_id),
    CHECK(other IS NULL OR json_valid(other))
);

CREATE TABLE meeting_times (
    sequence INTEGER,
    section_sequence TEXT,
    term_collection_id TEXT,
    subject_code TEXT,
    course_number TEXT,
    school_id TEXT,

    start_date TEXT,
    end_date TEXT,
    meeting_type TEXT,
    start_minutes TEXT,
    end_minutes TEXT,
    is_monday INTEGER NOT NULL CHECK(is_monday IN (0, 1)),
    is_tuesday INTEGER NOT NULL CHECK(is_tuesday IN (0, 1)),
    is_wednesday INTEGER NOT NULL CHECK(is_wednesday IN (0, 1)),
    is_thursday INTEGER NOT NULL CHECK(is_thursday IN (0, 1)),
    is_friday INTEGER NOT NULL CHECK(is_friday IN (0, 1)),
    is_saturday INTEGER NOT NULL CHECK(is_saturday IN (0, 1)),
    is_sunday INTEGER NOT NULL CHECK(is_sunday IN (0, 1)),
    other TEXT,

    FOREIGN KEY (section_sequence, term_collection_id, school_id, subject_code, course_number)
        REFERENCES sections(sequence, term_collection_id, school_id, subject_code, course_number) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (sequence, section_sequence, term_collection_id, subject_code, course_number, school_id),
    CHECK(other IS NULL OR json_valid(other))
);
