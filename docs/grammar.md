# ClassQL Grammar Specification

## Field Query Support Legend

- **V** - The field will be part of the large course information vector
- **N** - The field can only be queried manually  
- **"K"** - Where K is a keyword
- **{K}** - Is an optional keyword
- **|E|** - Is another expression defined elsewhere
- **||** - Is simply "or"
- **REPEATED** - Field can appear multiple times in queries

## Entity Field Definitions

term
    season ENUM of Spring Fall Winter Summer V

professor
    name                    V           "prof" {|condition|} <str>
    email_address           V           "prof" {|condition|} <str>
    first_name              V           "prof" {|condition|} <str>
    last_name               V           "prof" {|condition|} <str>
    other (JSON)            V            No?

courses
    subject_code            N           "subject" || "sub" {|condition|} <str> REPEATED
    number                  N           "course" {|condition|} <str>           REPEATED
    subject_description     V           "contains" || "subject description" {|condition|} <str>
    title                   V           "contains" || "title" <str>
    description             V           "contains" || "course desription" <str>
    credit_hours            N           "credit hours" {|binop|} <int>
    prerequisites           V           "prereqs" <str> | <strs>
    corequisites            V           "corereqs" <str> | <strs>
    other (JSON)            V            No?

sections
    subject_code            N           "subject" | "sub" {|condition|} <str> REPEATED
    course_number           N           "course" {|condition|} <str>          REPEATED
    max_enrollment          N           "cap" {|binop|} <int>
    instruction_method      V           "method" {|condition|} <str>
    campus                  V           "campus" {|condition|} <str>
    enrollment              N           "pop" {|binop|} <int>
    is_full? <custom>       N           {|condition|} "full"
    other (JSON)            V

meeting_times
    subject_code            N           "subject" | "sub" {|condition|} <str> REPEATED
    course_number           N           "course" {|condition|} <str>          REPEATED
    start_date              V           "start(s)" | "begin(s)" | "date" {|binop|} <str>
    end_date                V           "end(s)" | "date" {|binop|} <str>
    meeting_type            V           "meeting type" | "type" {|condition|} <str> ENUM?
    start_minutes           N           "time" {|binop|} <str>
    end_minutes             N           "time" {|binop|} <str>
    is_monday               N           "mon" | "monday" | "m" {|condition|} <str>
    is_tuesday              N           "tues" | "tuesday" | "tu" {|condition|} <str>
    is_wendesday            N           "wen" | "wendesday" | "w" {|condition|} <str>
    is_thursday             N           "thur" | "thursday" | "th" {|condition|} <str>
    is_friday               N           "fri" | "friday" | "f" {|condition|} <str>
    is_saturday             N           "sat" | "saturday" | "sa" {|condition|} <str>
    is_sunday               N           "sun" | "sunday" | "su" {|condition|} <str>
    other (JSON)            V

buildings
    name                    V           "building" {|condition|} <str>
    abbreviation            V           "building" {|condition|} <str>
    address                 V           "building" | "address" {|condition|} <str>
    other (JSON)            V

locations
    room_number             V           "room" | "room number" | "rm" {|condition|} <str>
    capacity                N            No?
    accessibility_features  V           "accessibility" {|condition|} <str>
    other (JSON)            V

## Formal BNF Grammar

```bnf
<query> ::= <entity_query> | <query> "or" <query>

<entity_query> ::= <term_query> | <professor_query> | <course_query> | <section_query> | <meeting_time_query> | <building_query> | <location_query>

<term_query> ::= "term" <condition> <season_value>
<season_value> ::= "Spring" | "Fall" | "Winter" | "Summer"

<professor_query> ::= <prof_name_query> | <prof_email_query> | <prof_first_name_query> | <prof_last_name_query>
<prof_name_query> ::= "prof" <condition> <string>
<prof_email_query> ::= "prof" <condition> <string>
<prof_first_name_query> ::= "prof" <condition> <string>
<prof_last_name_query> ::= "prof" <condition> <string>

<course_query> ::= <subject_query> | <course_number_query> | <course_title_query> | <course_description_query> | <credit_hours_query> | <prereqs_query> | <coreqs_query>
<subject_query> ::= ("subject" | "sub") <condition> <string>
<course_number_query> ::= "course" <condition> <string>
<course_title_query> ::= ("contains" | "title") <string>
<course_description_query> ::= ("contains" | "course description") <string>
<credit_hours_query> ::= "credit hours" <binop> <integer>
<prereqs_query> ::= "prereqs" <string> | "prereqs" <string_list>
<coreqs_query> ::= "corereqs" <string> | "corereqs" <string_list>

<section_query> ::= <section_subject_query> | <section_course_query> | <enrollment_cap_query> | <instruction_method_query> | <campus_query> | <enrollment_query> | <full_query>
<section_subject_query> ::= ("subject" | "sub") <condition> <string>
<section_course_query> ::= "course" <condition> <string>
<enrollment_cap_query> ::= "cap" <binop> <integer>
<instruction_method_query> ::= "method" <condition> <string>
<campus_query> ::= "campus" <condition> <string>
<enrollment_query> ::= "pop" <binop> <integer>
<full_query> ::= <condition> "full"

<meeting_time_query> ::= <meeting_subject_query> | <meeting_course_query> | <start_date_query> | <end_date_query> | <meeting_type_query> | <time_query> | <day_query>
<meeting_subject_query> ::= ("subject" | "sub") <condition> <string>
<meeting_course_query> ::= "course" <condition> <string>
<start_date_query> ::= ("start" | "starts" | "begin" | "begins" | "date") <binop> <string>
<end_date_query> ::= ("end" | "ends" | "date") <binop> <string>
<meeting_type_query> ::= ("meeting type" | "type") <condition> <string>
<time_query> ::= "time" <binop> <string>
<day_query> ::= <monday_query> | <tuesday_query> | <wednesday_query> | <thursday_query> | <friday_query> | <saturday_query> | <sunday_query>
<monday_query> ::= ("mon" | "monday" | "m") <condition> <string>
<tuesday_query> ::= ("tues" | "tuesday" | "tu") <condition> <string>
<wednesday_query> ::= ("wen" | "wednesday" | "w") <condition> <string>
<thursday_query> ::= ("thur" | "thursday" | "th") <condition> <string>
<friday_query> ::= ("fri" | "friday" | "f") <condition> <string>
<saturday_query> ::= ("sat" | "saturday" | "sa") <condition> <string>
<sunday_query> ::= ("sun" | "sunday" | "su") <condition> <string>

<building_query> ::= <building_name_query> | <building_abbrev_query> | <building_address_query>
<building_name_query> ::= "building" <condition> <string>
<building_abbrev_query> ::= "building" <condition> <string>
<building_address_query> ::= ("building" | "address") <condition> <string>

<location_query> ::= <room_query> | <accessibility_query>
<room_query> ::= ("room" | "room number" | "rm") <condition> <string>
<accessibility_query> ::= "accessibility" <condition> <string>

<condition> ::= "=" | "!=" | "contains" | "starts_with" | "ends_with"
<binop> ::= "=" | "!=" | "<" | ">" | "<=" | ">=" 

<string> ::= '"' [^"]* '"' | [a-zA-Z0-9_]+
<integer> ::= [0-9]+
<string_list> ::= <string> | <string_list> "," <string>
```