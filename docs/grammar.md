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
    prerequisites           V           "prereqs" {|condition|} <str>
    corequisites            V           "corereqs" {|condition|} <str>
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

## Lexical Patterns

The following regex patterns define how the lexer tokenizes ClassQL input:

### Special Identifiers
- **Email-like identifiers**: `[a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_]*\.[a-zA-Z0-9_.]*`
  - Example: `prof@email.com` → Single `T_IDENTIFIER` token

### Keywords
- **Subject**: `\b(subject|sub)\b` → `T_SUBJECT`
- **Course**: `\bcourse\b` → `T_COURSE` 
- **Professor**: `\bprof\b` → `T_PROF`
- **Days**: Progressive abbreviations with word boundaries
  - Monday: `\b(monday|monda|mond|mon|mo|m)\b` → `T_MONDAY`
  - Tuesday: `\b(tuesday|tuesda|tuesd|tues|tue|tu)\b` → `T_TUESDAY`
  - Wednesday: `\b(wednesday|wednesda|wednesd|wednes|wedne|wedn|wed|we|w)\b` → `T_WEDNESDAY`
  - Thursday: `\b(thursday|thursda|thurs|thur|thu|th)\b` → `T_THURSDAY`
  - Friday: `\b(friday|frida|frid|fri|fr|f)\b` → `T_FRIDAY`
  - Saturday: `\b(saturday|saturda|saturd|satur|satu|sat|sa)\b` → `T_SATURDAY`
  - Sunday: `\b(sunday|sunda|sund|sun|su)\b` → `T_SUNDAY`

### Operators
- **Comparison**: `!=`, `<=`, `>=`, `=`, `<`, `>`, `!`
- **Logical**: `\band\b`, `\bor\b`, `\bnot\b`
- **Grouping**: `\(`, `\)`

### Literals
- **Strings**: `"[^"]*"?` (supports unclosed strings)
- **Times**: `[0-9]+:[0-9]+\s?(?:am|pm)|[0-9]+\s?(?:am|pm)` (am/pm suffix required)
- **Alphanumeric**: `[0-9]+[A-Za-z]+` (course numbers like "424N", "101L" - digits followed by letters)
- **Integers**: `[0-9]+`
- **General Identifiers**: `[a-zA-Z_][a-zA-Z0-9_]*`

### Tokenization Order
1. Email-like identifiers (highest priority)
2. Multi-word operators
3. Keywords
4. Day abbreviations
5. Single operators
6. Logical operators
7. Conditions
8. Literals (strings, times, alphanumeric, integers)
9. General identifiers (lowest priority)

## Formal BNF Grammar

```bnf
<query> ::= <logical_term> ("or" <logical_term>)*

<logical_term> ::= <logical_factor> ("and" <logical_factor>)*

<logical_factor> ::= <entity_query> | "(" <query> ")" | "not" <logical_factor>

<entity_query> ::= <professor_query> | <course_query> | <section_query> | <meeting_type_query> | <time_query> | <day_query>

<professor_query> ::= "prof" <condition> <string>

<course_query> ::= "course" (<condition> <string> | <subject_query> | <number_query> | <title_query> | <description_query> | <credit_hours_query> | <prereqs_query> | <corereqs_query>)
<subject_query> ::= ("subject" | "sub") <condition> <string>
<number_query> ::= "number" <condition> <string>
<title_query> ::= "title" <condition> <string>
<description_query> ::= "description" <condition> <string>
<credit_hours_query> ::= "credit hours" <binop> <integer>
<prereqs_query> ::= "prereqs" <condition> <string>
<corereqs_query> ::= "corereqs" <condition> <string>

<section_query> ::= "section" <subject_query> | <course_query> | <enrollment_cap_query> | <instruction_method_query> | <campus_query> | <enrollment_query> | <full_query>
<enrollment_query> ::= "size" <binop> <integer> | "enrollment" <binop> <integer>
<enrollment_cap_query> ::= "enrollment cap" <binop> <integer> | "cap" <binop> <integer>
<instruction_method_query> ::= "method" <condition> <string>
<campus_query> ::= "campus" <condition> <string>
<full_query> ::= "full" <condition> <string>

<meeting_type_query> ::= ("meeting type" | "type") <condition> <string>
<time_query> ::= ("start" | "end") (<binop> <time> | <time_range>)
<time_range> ::= <time> "to" <time>
<day_query> ::= <monday_query> | <tuesday_query> | <wednesday_query> | <thursday_query> | <friday_query> | <saturday_query> | <sunday_query>
<monday_query> ::= ("mon" | "monday" | "m") [<condition> <string>]
                    If condition is omitted, defaults to "= true"
<tuesday_query> ::= ("tues" | "tuesday" | "tu") [<condition> <string>]
                     If condition is omitted, defaults to "= true"
<wednesday_query> ::= ("wen" | "wednesday" | "w") [<condition> <string>]
                      If condition is omitted, defaults to "= true"
<thursday_query> ::= ("thur" | "thursday" | "th") [<condition> <string>]
                     If condition is omitted, defaults to "= true"
<friday_query> ::= ("fri" | "friday" | "f") [<condition> <string>]
                    If condition is omitted, defaults to "= true"
<saturday_query> ::= ("sat" | "saturday" | "sa") [<condition> <string>]
                      If condition is omitted, defaults to "= true"
<sunday_query> ::= ("sun" | "sunday" | "su") [<condition> <string>]
                    If condition is omitted, defaults to "= true"

<time> ::= [0-9]+:[0-9]+\s?(?:am|pm)|[0-9]+\s?(?:am|pm)  ; am/pm suffix required
<condition> ::= "=" | "!=" | "contains" | "has" | "starts with" | "ends with" | "is" | "equals" | "not equals" | "does not equal"
<binop> ::= "=" | "!=" | "<" | ">" | "<=" | ">=" | "equals" | "is" | "not equals" | "not" | "does not equal" | "less than" | "greater than" | "less than or equal to" | "greater than or equal to" | "at least" | "at most" | "more than" | "fewer than"

<string> ::= "[^"]*"?
<integer> ::= [0-9]+
<identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*
<email_identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_]*\.[a-zA-Z0-9_.]*
<string_list> ::= <string> | <string_list> "," <string>
```