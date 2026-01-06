# Test Queries for ClassQL

This document provides a comprehensive list of test queries and their expected results.

## Queries with Descriptions (Test Dynamic Height)

### 1. CMPT 424N - Long Description
**Query:** `prof contains Alan` or `subject = CMPT and course = 424N`
**Expected Results:**
- **CMPT 424N-111** - OPERATING SYSTEMS
  - Professor: Labouseur, Alan
  - Email: Alan.Labouseur@marist.edu
  - **Has description** (566 chars) - should show expanded detail view

### 2. ACCT 203N - Medium Description
**Query:** `subject = ACCT and course = 203N` or `prof contains Craven`
**Expected Results:**
- **ACCT 203N-111** - FINANCIAL ACCTNG (Craven, Michael)
- **ACCT 203N-113** - FINANCIAL ACCTNG (Craven, Michael)
- **Has description** (198 chars) - should show moderately expanded detail view

## Basic Professor Queries

### 3. Simple Professor Name
**Query:** `prof contains Alan`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS (Labouseur, Alan)
- Any other sections taught by professors with "Alan" in their name

### 4. Professor with "is" operator
**Query:** `prof is Labouseur`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS (Labouseur, Alan)

### 5. Professor with "has" operator
**Query:** `prof has Ardalan`
**Expected Results:**
- BUS 320N-115 - FINANCIAL MGMT (Ardalan, Kavous)
- BUS 320N-117 - FINANCIAL MGMT (Ardalan, Kavous)

### 6. Professor Email Search
**Query:** `prof contains marist.edu`
**Expected Results:**
- Multiple sections (professors with marist.edu email addresses)

## Subject and Course Queries

### 7. Subject Code Exact Match
**Query:** `subject = CMPT`
**Expected Results:**
- All CMPT (Computer Science) courses and sections

### 8. Subject Code with "contains"
**Query:** `subject contains CMP`
**Expected Results:**
- All CMPT courses (and any other subjects containing "CMP")

### 9. Course Number
**Query:** `course = 424N`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS
- Any other sections of course 424N

### 10. Subject and Course Combined
**Query:** `subject = CMPT and course = 424N`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS

### 11. Subject Abbreviation
**Query:** `sub = BUS`
**Expected Results:**
- All BUS (Business) courses and sections

## Title and Description Queries

### 12. Title Contains
**Query:** `title contains OPERATING`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS

### 13. Title Contains (Partial)
**Query:** `title contains SYSTEMS`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS
- Any other courses with "SYSTEMS" in title

### 14. Description Contains
**Query:** `description contains operating`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS (has description with "operating")
- ACCT 203N sections (has description with "accounting")

## Credit Hours Queries

### 15. Credit Hours Greater Than
**Query:** `credit hours > 3`
**Expected Results:**
- All courses with more than 3 credit hours

### 16. Credit Hours Equals
**Query:** `credit hours = 4`
**Expected Results:**
- CMPT 424N-111 (4 credits)
- All other 4-credit courses

### 17. Credit Hours Less Than
**Query:** `credit hours < 3`
**Expected Results:**
- All courses with less than 3 credit hours

## Meeting Type Queries

### 18. Lecture Type
**Query:** `type = LEC`
**Expected Results:**
- CMPT 424N-111 (LEC)
- All other lecture sections

### 19. Lab Type
**Query:** `type = LAB`
**Expected Results:**
- All laboratory sections

### 20. Web/Online Type
**Query:** `type = WEB`
**Expected Results:**
- All web/online sections

## Day Queries

### 21. Monday Classes
**Query:** `monday`
**Expected Results:**
- All sections that meet on Monday
- BUS 320N-115 (MW 3:30pm-4:45pm)
- BUS 320N-117 (MW 5:00pm-6:15pm)

### 22. Thursday Classes
**Query:** `thursday` or `th`
**Expected Results:**
- CMPT 424N-111 (R 8:00am-9:15am, M 8:00am-10:45am)
- All other Thursday sections

### 23. Monday and Wednesday
**Query:** `monday and wednesday`
**Expected Results:**
- BUS 320N-115 (MW 3:30pm-4:45pm)
- BUS 320N-117 (MW 5:00pm-6:15pm)
- All other MW sections

## Time Queries

### 24. Start Time After
**Query:** `start > 2pm`
**Expected Results:**
- All sections starting after 2:00 PM
- BUS 320N-117 (MW 5:00pm-6:15pm)

### 25. Start Time Before
**Query:** `start < 10am`
**Expected Results:**
- CMPT 424N-111 (R 8:00am-9:15am, M 8:00am-10:45am)
- All other sections starting before 10:00 AM

### 26. End Time Before
**Query:** `end < 12pm`
**Expected Results:**
- All sections ending before noon

## Enrollment Queries

### 27. Enrollment Size
**Query:** `size = 0`
**Expected Results:**
- All sections with 0 enrolled students
- Most sections in the database

### 28. Enrollment Cap
**Query:** `cap = 25`
**Expected Results:**
- CMPT 424N-111 (cap: 25)
- ACCT 203N-111, 112, 113, 114 (cap: 25)
- All other sections with cap of 25

### 29. Enrollment Greater Than
**Query:** `enrollment > 0`
**Expected Results:**
- All sections with at least 1 enrolled student

## Complex Combined Queries

### 30. Professor and Subject
**Query:** `prof contains Alan and subject = CMPT`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS (Labouseur, Alan)

### 31. Subject, Course, and Type
**Query:** `subject = CMPT and course = 424N and type = LEC`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS (if it's a LEC)

### 32. Title and Credit Hours
**Query:** `title contains SYSTEMS and credit hours = 4`
**Expected Results:**
- CMPT 424N-111 - OPERATING SYSTEMS (4 credits)

### 33. Professor OR Subject
**Query:** `prof contains Alan or subject = ACCT`
**Expected Results:**
- CMPT 424N-111 (Labouseur, Alan)
- All ACCT courses

### 34. NOT Operator
**Query:** `not subject = CMPT`
**Expected Results:**
- All courses EXCEPT CMPT courses

### 35. Complex AND/OR
**Query:** `(subject = CMPT or subject = BUS) and credit hours > 3`
**Expected Results:**
- CMPT 424N-111 (4 credits)
- All BUS courses with more than 3 credits

## Campus Queries

### 36. Campus Contains
**Query:** `campus contains Marist`
**Expected Results:**
- CMPT 424N-111 (Marist College Campus)
- All other sections at Marist College Campus

## Method Queries

### 37. Instruction Method
**Query:** `method contains Online`
**Expected Results:**
- All sections with "Online" in instruction method

## Queries with No Results

### 38. Non-existent Professor
**Query:** `prof is Nonexistent`
**Expected Results:**
- No results (empty list)

### 39. Non-existent Course
**Query:** `subject = CMPT and course = 9999`
**Expected Results:**
- No results (empty list)

### 40. Impossible Combination
**Query:** `subject = CMPT and subject = BUS`
**Expected Results:**
- No results (empty list)

## Testing Dynamic Height Feature

**To test the dynamic height feature:**

1. **Query with description:** `prof contains Alan`
   - Open detail view for CMPT 424N-111
   - Should see expanded height with full description

2. **Query with description:** `subject = ACCT and course = 203N`
   - Open detail view for ACCT 203N-111 or 113
   - Should see moderately expanded height with description

3. **Query without description:** `prof contains Ardalan`
   - Open detail view for BUS 320N-115 or 117
   - Should see compact height with "(No description available)"

4. **Compare heights:**
   - Open CMPT 424N-111 (long description) - should be near max height
   - Open ACCT 203N-111 (medium description) - should be medium height
   - Open BUS 320N-115 (no description) - should be minimum height

