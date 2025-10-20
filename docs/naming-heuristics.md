# Intelligent Naming Heuristics

This document describes the smart heuristics Nameback uses to generate meaningful filenames from file metadata and content.

## Table of Contents

- [Philosophy](#philosophy)
- [Current Implementation](#current-implementation)
- [Quality Scoring System](#quality-scoring-system)
- [Metadata Filtering](#metadata-filtering)
- [Content Extraction Strategies](#content-extraction-strategies)
- [Planned Enhancements](#planned-enhancements)
- [Examples](#examples)

---

## Philosophy

Nameback's naming philosophy prioritizes:

1. **Meaningful over generic** - A descriptive name beats a timestamp
2. **Quality over quantity** - Better to keep original than generate garbage
3. **Context awareness** - Use all available information (metadata, content, location, structure)
4. **User control** - Dry-run mode and clear logging for transparency
5. **Safety first** - Never overwrite existing files, validate permissions

---

## Current Implementation

### Priority-Based Name Selection

Nameback extracts names from metadata fields in priority order based on file category:

| File Category | Priority Order |
|---------------|----------------|
| **Image** | Title → Description → DateTimeOriginal |
| **Document** | Title → Subject → Author |
| **Audio** | Title → Artist → Album |
| **Video** | Title → CreationDate |

### Fallback Content Extraction

When metadata is missing or unhelpful, Nameback extracts content directly:

- **PDFs**: Extract text using `pdf_extract`, fallback to OCR if needed
- **Plain Text**: Read file content and extract meaningful first line
- **Images**: Use Tesseract OCR (Traditional Chinese, Simplified Chinese, English)
- **Videos**: Extract frame at 00:00:05 and OCR

### Current Metadata Filtering

The `is_useful_metadata()` function filters out unhelpful values:

```rust
fn is_useful_metadata(value: &Option<String>) -> bool {
    if let Some(v) = value {
        let lower = v.to_lowercase();

        // Skip if:
        if lower.is_empty() { return false; }
        if lower.len() < 3 { return false; }
        if lower.contains("canon") { return false; }
        if lower.contains("printer") { return false; }
        if lower.contains("scanner") { return false; }
        if lower.contains("ipr") { return false; }
        if lower.contains("untitled") { return false; }

        true
    } else {
        false
    }
}
```

**Filters out:**
- Empty strings
- Very short strings (< 3 characters)
- Scanner/printer device names
- Generic placeholders ("Untitled")

---

## Quality Scoring System

### Concept (Planned)

Instead of simple priority-based selection, score each candidate name on multiple dimensions:

```rust
score = length_score * 2.0
      + source_reliability
      + word_count_bonus
      + diversity_bonus
      - penalty_factors
```

### Scoring Dimensions

#### 1. Length Score (0.0 - 1.0)
Optimal length is 20-60 characters:

| Length | Score | Rationale |
|--------|-------|-----------|
| 0-10 | 0.2 | Too short, lacks context |
| 11-19 | 0.6 | Short but acceptable |
| 20-60 | 1.0 | **Optimal range** |
| 61-100 | 0.7 | Long but manageable |
| 100+ | 0.4 | Too long, hard to read |

#### 2. Source Reliability (0.5 - 3.0)

| Source | Score | Rationale |
|--------|-------|-----------|
| File Metadata (EXIF) | 3.0 | Highest quality, author-provided |
| Text Extraction | 2.5 | Direct content, usually accurate |
| PDF Text | 2.0 | Good quality but may include noise |
| OCR from Image | 1.5 | Prone to errors, needs validation |
| Fallback/Generated | 0.5 | Last resort |

#### 3. Word Count Bonus (0.0 - 2.5)

```
bonus = min(word_count, 5) * 0.5
```

- Encourages multi-word descriptive names
- Caps at 5 words to avoid long sentences
- Examples:
  - "report" (1 word) → 0.5 bonus
  - "quarterly sales report" (3 words) → 1.5 bonus
  - "the quick brown fox jumps" (5 words) → 2.5 bonus
  - "the quick brown fox jumps over" (6 words) → 2.5 bonus (capped)

#### 4. Character Diversity (0.0 - 1.5)

```
diversity = unique_chars / total_chars
bonus = diversity * 1.5
```

- Penalizes repetitive text ("AAAAAAA", "1111111")
- Rewards varied, meaningful content
- Examples:
  - "aaaaaa" → diversity = 1/6 = 0.17 → 0.25 bonus
  - "Invoice_2024" → diversity = 11/12 = 0.92 → 1.38 bonus

#### 5. Penalty Factors

**Date-Only Names** (×0.3 multiplier)
- Pattern: YYYY-MM-DD, MM/DD/YYYY without context
- Examples: "2023-01-15", "01-15-2023"
- Rationale: Not descriptive, should be suffix not primary name

**Error Indicators** (×0.2 multiplier)
- Contains: "error", "exception", "warning", "failed", "traceback"
- Example: "ERROR: Cannot read file"
- Rationale: OCR/extraction failure artifacts

**Technical IDs** (×0.3 multiplier)
- UUIDs, hashes, serial numbers
- Examples: "a3d5e7f9-1234-5678", "MD5:abc123"
- Rationale: Not human-readable

### Example Scoring

**Candidate A**: "IMG_20231015_143022" (from OCR)
```
length_score: 0.6 (19 chars)
source_reliability: 1.5 (OCR)
word_count: 0.5 (1 word)
diversity: 0.95 (high variety)
penalty: none

Total: (0.6 * 2.0) + 1.5 + 0.5 + 0.95 = 4.15
```

**Candidate B**: "Quarterly Sales Report Q3 2023" (from metadata)
```
length_score: 1.0 (31 chars, optimal)
source_reliability: 3.0 (metadata)
word_count: 2.5 (5 words)
diversity: 1.35 (high variety)
penalty: none

Total: (1.0 * 2.0) + 3.0 + 2.5 + 1.35 = 8.85
```

**Winner**: Candidate B (8.85 vs 4.15) - Clear quality difference!

### Confidence Thresholds

Use scoring to decide whether to rename:

| Score | Action | Rationale |
|-------|--------|-----------|
| < 2.0 | Keep original filename | Quality too low |
| 2.0 - 5.0 | Rename with caution | Marginal improvement |
| 5.0 - 8.0 | Rename confidently | Good quality name |
| > 8.0 | Rename (excellent) | High quality name |

---

## Metadata Filtering

### Enhanced Filter Patterns (Planned)

Expand beyond current basic filtering to catch more unhelpful patterns:

#### Error Messages and Warnings
```
error, exception, warning, failed, cannot, invalid,
undefined, null, errno, traceback, fatal, critical
```

**Examples to reject:**
- "ERROR: Cannot read metadata"
- "Warning: Invalid timestamp"
- "Exception in module foo"

#### Scanner and Device Names
```
canon, printer, scanner, ipr, epson, hp, brother,
xerox, kyocera, ricoh, lexmark, dell, fujitsu
```

**Examples to reject:**
- "Canon MX490 Series"
- "HP LaserJet 1200"
- "EPSON Scanner"

#### Generic Placeholder Text
```
untitled, new document, document1, image1, noname,
unnamed, temp, test, sample, copy of, draft, file
```

**Examples to reject:**
- "Untitled Document"
- "New Document 1"
- "Image1"
- "Copy of Report"
- "test_file"

#### Date-Only Strings
```
Pattern: ^\d{4}[-/]\d{2}[-/]\d{2}$
Pattern: ^\d{2}[-/]\d{2}[-/]\d{4}$
Pattern: ^\d{8}$
```

**Examples to reject:**
- "2023-01-15"
- "01/15/2023"
- "20230115"

**Allow when contextualized:**
- "Meeting Notes 2023-01-15" ✓
- "Invoice_2023_Q1" ✓

#### Technical Identifiers
```
Pattern: UUID format (8-4-4-4-12 hex digits)
Pattern: Hash-like (long hex/base64 strings)
Pattern: Serial numbers (model-numbers with dashes/underscores)
```

**Examples to reject:**
- "a3d5e7f9-1234-5678-90ab-cdef12345678"
- "MD5:7d793037a0760186574b0282f2f435e7"
- "SN-12345-ABCD-9876"

#### Excessive Repetition

Detect strings with too many repeated characters:

```rust
fn has_excessive_repetition(s: &str) -> bool {
    let max_repeat = 3;
    let mut prev = '\0';
    let mut count = 0;

    for ch in s.chars() {
        if ch == prev {
            count += 1;
            if count > max_repeat { return true; }
        } else {
            count = 1;
            prev = ch;
        }
    }
    false
}
```

**Examples to reject:**
- "aaaaaa"
- "1111111"
- "----test----"

**Allow:**
- "coffee" (double 'f' and 'e' are normal)
- "bookkeeper" (legitimate repeated letters)

#### Mostly Punctuation/Symbols

Reject if alphanumeric content is less than 1/3 of total:

```rust
let alpha_count = v.chars().filter(|c| c.is_alphanumeric()).count();
if alpha_count < v.len() / 3 { return false; }
```

**Examples to reject:**
- "!!!###$$$"
- "---***---"
- "..::..::."

**Allow:**
- "C++ Programming" (mostly alpha)
- "user@example.com" (enough alpha content)

---

## Content Extraction Strategies

### Plain Text Files

**Current**: Extract first line > 10 characters

**Enhanced**:
1. **Skip common prefixes**: TODO:, FIXME:, ///, #, /\*, REM
2. **Detect title patterns**:
   - ALL CAPS lines (likely headers)
   - Underlined text (= or - beneath)
   - Lines with title case (First Letter Capitalized)
3. **Skip code patterns**:
   - Lines with `;`, `{`, `}`, `()` patterns
   - Lines starting with `import`, `function`, `def`, `class`
4. **Handle structured formats**:
   - Org-mode: Extract `#+TITLE:`
   - reStructuredText: First `===` or `---` header
   - AsciiDoc: Extract `= Title` syntax

**Examples:**

```text
Input File: meeting_notes.txt
---
TODO: Review Q3 results

Q3 Sales Meeting
================

Today we discussed...
---
Extracted Name: "Q3_Sales_Meeting"
```

### Markdown Files

**Current**: Extract first `#` heading

**Enhanced**:
1. **Parse frontmatter** (YAML/TOML):
   ```yaml
   ---
   title: My Great Article
   author: John Doe
   ---
   ```
   Use `title` field if present

2. **Skip generic headings**:
   - "Introduction", "Overview", "Table of Contents"
   - "Chapter 1" without context
   - "Summary", "Conclusion"

3. **Combine heading hierarchy**:
   ```markdown
   # User Guide
   ## Chapter 3: Advanced Features
   ```
   Result: "User_Guide_Chapter_3_Advanced_Features"

**Examples:**

```markdown
Input: technical_docs.md
---
# Documentation

## Table of Contents

## Installation Guide

Follow these steps...
---
Skip: "Documentation" (too generic)
Skip: "Table of Contents" (generic)
Use: "Installation_Guide"
```

### CSV Files

**Current**: Concatenate first 3 columns

**Enhanced**:
1. **Detect header row**: Check for column names
2. **Prioritize semantic columns**:
   - Look for: "name", "title", "description", "subject"
   - Skip: "id", "timestamp", "created_at", "index"
3. **Use only text columns**:
   - Skip numeric-only columns
   - Skip date columns
4. **Limit to 2 columns** for cleaner names
5. **Handle quoted CSV** properly

**Examples:**

```csv
Input: contacts.csv
---
id,name,email,created_at
1,John Doe,john@example.com,2023-01-15
---
Skip: "id" (identifier)
Use: "name" → "John_Doe"
Skip: "email" (secondary)
Skip: "created_at" (date)
Result: "John_Doe"
```

```csv
Input: inventory.csv
---
product_id,product_name,category,quantity,price
12345,Ergonomic Keyboard,Electronics,150,79.99
---
Skip: "product_id" (id)
Use: "product_name" → "Ergonomic_Keyboard"
Use: "category" → "Electronics"
Skip: "quantity" (numeric)
Skip: "price" (numeric)
Result: "Ergonomic_Keyboard_Electronics"
```

### JSON/YAML Files

**Current**: Top-level fields only (`title`, `name`, `description`)

**Enhanced**:
1. **Deep nested search**:
   ```json
   {
     "data": {
       "metadata": {
         "title": "Project Apollo"
       }
     }
   }
   ```
   Search: `data.metadata.title`, `metadata.title`, `title`

2. **Semantic field prioritization**:
   - Priority 1: `title`, `name`
   - Priority 2: `displayName`, `label`, `heading`
   - Priority 3: `description`, `summary`
   - Priority 4: `id` (only if meaningful, not UUID)

3. **Handle arrays**:
   ```json
   {
     "items": [
       {"title": "First Item"},
       {"title": "Second Item"}
     ]
   }
   ```
   Use first element: "First_Item"

4. **Format-specific parsing**:
   - **package.json**: Combine `name` + `description`
   - **composer.json**: Same pattern
   - **pyproject.toml**: Extract `[project].name`
   - **config files**: Look for `app_name`, `project_name`, `site_title`

**Examples:**

```json
Input: package.json
---
{
  "name": "my-awesome-app",
  "version": "1.0.0",
  "description": "A tool for organizing files"
}
---
Result: "my_awesome_app_A_tool_for_organizing_files"
```

```yaml
Input: config.yaml
---
server:
  host: localhost
  port: 8080
application:
  name: File Processor
  version: 2.1
---
Result: "File_Processor"
```

### PDF Files

**Current**: Extract all text, truncate to 80 chars

**Enhanced**:
1. **Detect document structure**:
   - Skip headers/footers (repeated text)
   - Identify title (large font, first significant text)
   - Skip page numbers
2. **Extract key phrases** instead of truncating:
   - Use n-gram extraction
   - Score by position and frequency
   - Return top 3 phrases
3. **OCR fallback** when text extraction fails:
   - Current: OCR entire first page
   - Enhanced: OCR only title region (top 30% of page)

**Examples:**

```
Input: research_paper.pdf
---
Page text:
"
A Comprehensive Study of File Organization Systems
John Smith, University of XYZ
Published: 2023

Abstract: This paper presents...
"
---
Current: "A_Comprehensive_Study_of_File_Organization_Systems_John_Smith_Univ"
Enhanced: "Comprehensive_Study_File_Organization_Systems" (key phrases)
```

### Image Files (OCR)

**Current**: OCR entire image with Tesseract (3 languages)

**Enhanced**:
1. **Quality pre-filtering**:
   - Check image dimensions (skip < 100x100)
   - Check file size (skip < 10KB - likely icon/thumbnail)
2. **Preprocessing for better OCR**:
   - Contrast enhancement
   - Deskew rotation correction
   - Noise reduction
3. **Region-based OCR**:
   - Detect text regions first
   - OCR only text-heavy regions
   - Skip background/decorative areas
4. **Post-processing**:
   - Filter out OCR noise (single chars, gibberish)
   - Combine text blocks intelligently
   - Score results and keep best

**Examples:**

```
Input: screenshot.png
---
Image contains:
- Top banner: "MyApp v2.1"
- Main area: "Error: Connection timeout"
- Bottom: "© 2023 Company Inc"
---
Current: "MyApp_v2_1_Error_Connection_timeout_2023_Company_Inc"
Enhanced: "MyApp_v2_1" (ignore error message, skip copyright)
```

### Video Files

**Current**: Extract frame at 00:00:05, OCR

**Enhanced**:
1. **Multi-frame sampling**:
   - Extract at: 00:00:01, 00:00:05, 00:00:10
   - OCR each frame
   - Score results, pick best
2. **Container metadata first**:
   - Use `ffprobe` to extract title from container
   - Check metadata streams
   - Prefer container title over OCR
3. **Scene detection**:
   - Detect scene changes (title cards)
   - Extract frames at scene boundaries
   - Higher chance of meaningful text

**Examples:**

```
Input: presentation.mp4
---
Frame at 00:00:01: "Loading..."
Frame at 00:00:05: "2023 Q4 Results Presentation"
Frame at 00:00:10: "Agenda"
Container metadata: None
---
Current: "Loading" (from 00:00:05)
Enhanced: "2023_Q4_Results_Presentation" (best scoring frame)
```

---

## Planned Enhancements

### Phase 1: Quick Wins

#### A. Quality Scoring System
See [Quality Scoring System](#quality-scoring-system) section above.

**Implementation**: Add `src/scorer.rs` module

#### B. Enhanced Error/Noise Filtering
See [Metadata Filtering](#metadata-filtering) section above.

**Implementation**: Expand `is_useful_metadata()` in `src/extractor.rs`

#### C. Intelligent Filename Stem Analysis

Extract meaningful parts from original cryptic filenames:

```rust
fn extract_meaningful_parts_from_original(path: &Path) -> Option<String> {
    // "IMG_20231015_143022_HDR.jpg" → "143022_HDR"
    // "Screenshot_2023-10-15_at_14.30.22.png" → None (all date/time)
    // "Project_Proposal_FINAL_v3_FINAL.docx" → "Project_Proposal"
}
```

**Remove common prefixes:**
- IMG_, DSC_, SCAN_, Screenshot_, Capture_, VID_
- Copy_of_, Draft_, New_, Untitled_

**Filter out parts:**
- Pure numeric sequences (likely timestamps)
- Date patterns (YYYYMMDD, YYYY-MM-DD)
- Version numbers (v1, v2, final, final2)

**Implementation**: Add to `src/extractor.rs`

### Phase 2: Content Intelligence

#### D. Directory Context

```rust
fn extract_context_from_path(path: &Path) -> Option<String> {
    // /Users/john/Projects/Website/images/hero.jpg
    // → "Website_images" (skip "Projects" as generic)

    // /home/user/Invoices/2023/Q4/invoice.pdf
    // → "Invoices_2023_Q4"
}
```

**Skip generic directories:**
- documents, downloads, desktop, files
- tmp, temp, data, misc, other, new

**Implementation**: Add to `src/extractor.rs`

#### E. Format-Specific Intelligence

Add specialized extractors for:

- **Email (.eml, .msg)**: Subject - From - Date
- **Web archives (.html, .mhtml)**: `<title>` tag
- **Archives (.zip, .tar)**: Main content or manifest

**Implementation**: Add `src/format_extractors/` module

#### F. Key Phrase Extraction

Lightweight NLP without heavy ML dependencies:

```rust
fn extract_key_phrases(text: &str, max_phrases: usize) -> Vec<String> {
    // 1. Tokenize, remove stop words
    // 2. Generate n-grams (1-3 words)
    // 3. Score by frequency + position (earlier = better)
    // 4. Return top N phrases
}
```

**Stop words to filter:**
```
the, a, an, and, or, but, in, on, at, to, for, of,
with, by, from, as, is, was, are, were, been, be
```

**Implementation**: Add to `src/nlp.rs` module

### Phase 3: Performance & Polish

#### G. Metadata Caching

Cache expensive operations (OCR, exiftool):

```
~/.cache/nameback/metadata_cache.json
{
  "/path/to/file.jpg": {
    "mtime": "2023-10-15T14:30:22Z",
    "metadata": { "title": "Extracted Title", ... },
    "extracted_at": "2023-10-15T15:00:00Z"
  }
}
```

**Invalidation**: Check file mtime, discard if modified

**Implementation**: Add `src/cache.rs` module, use `dirs` crate

#### H. Multi-Frame Video Analysis

Already described in [Video Files](#video-files) section.

**Implementation**: Enhance `src/video_ocr.rs`

### Phase 4: Advanced Features

#### I. Series Detection

Detect related files and maintain consistency:

```
Input:
- screenshot_1.png
- screenshot_2.png
- screenshot_3.png

Detection: Series "screenshot_" with 3 members
Action: Keep sequential numbering
```

**Implementation**: Add `src/series_detector.rs` module

#### J. Location Enrichment

Use GPS metadata for location-based names:

```
EXIF GPS: 40.7128° N, 74.0060° W
Result: "Photo_NewYork_2023_10_15"
```

**Implementation**: Enhance `src/extractor.rs`, optionally add reverse geocoding

---

## Examples

### Example 1: Image with Poor OCR

**File**: `IMG_20231015.jpg`

**Current Flow**:
1. Detect: Image
2. Extract metadata: None (no EXIF title/description)
3. Fallback: OCR → "|||III|I|I|||" (scanning artifact)
4. Use: "|||III|I|I|||.jpg" ❌

**Enhanced Flow**:
1. Detect: Image
2. Extract metadata: None
3. Fallback: OCR → "|||III|I|I|||"
4. Score: 0.8 (low diversity, mostly punctuation)
5. Reject: Score < 2.0 threshold
6. Stem analysis: "IMG_20231015" → "20231015" (date only, reject)
7. Use: Original filename ✓

### Example 2: PDF Report

**File**: `document.pdf`

**Current Flow**:
1. Detect: Document
2. Extract metadata: Title = None
3. Fallback: Extract text → "Quarterly Sales Report Q3 2023 Prepared by Marketing Department This report summarizes..."
4. Truncate: "Quarterly_Sales_Report_Q3_2023_Prepared_by_Marketing_Department_Th"

**Enhanced Flow**:
1. Detect: Document
2. Extract metadata: Title = None
3. Fallback: Extract text (full content)
4. Key phrase extraction:
   - N-grams: "Quarterly Sales Report", "Q3 2023", "Marketing Department", ...
   - Score by position + frequency
   - Top phrases: "Quarterly Sales Report", "Q3 2023", "Prepared by"
5. Combine: "Quarterly_Sales_Report_Q3_2023"
6. Score: 8.5 (optimal length, high quality)
7. Use: "Quarterly_Sales_Report_Q3_2023.pdf" ✓

### Example 3: Video with Title Card

**File**: `VID_20231015_143022.mp4`

**Current Flow**:
1. Detect: Video
2. Extract metadata: Title = None
3. Fallback: Extract frame at 00:00:05
4. OCR: "Loading..." (loading screen)
5. Use: "Loading.mp4" ❌

**Enhanced Flow**:
1. Detect: Video
2. Extract metadata: Title = None
3. Multi-frame extraction:
   - 00:00:01 → OCR: "Loading..."
   - 00:00:05 → OCR: "Company Presentation 2023"
   - 00:00:10 → OCR: "Agenda"
4. Score results:
   - "Loading..." → 2.1 (low quality, generic)
   - "Company Presentation 2023" → 7.8 (high quality)
   - "Agenda" → 3.5 (short but okay)
5. Use best: "Company_Presentation_2023.mp4" ✓

### Example 4: Markdown with Frontmatter

**File**: `notes.md`

**Current Flow**:
1. Detect: Document
2. Extract metadata: Title = None
3. Content extraction: First heading = "Table of Contents"
4. Use: "Table_of_Contents.md" ❌

**Enhanced Flow**:
1. Detect: Document
2. Parse frontmatter:
   ```yaml
   ---
   title: Meeting Notes - Q3 Planning
   date: 2023-10-15
   ---
   ```
3. Extract: title = "Meeting Notes - Q3 Planning"
4. Score: 8.2 (from metadata, high quality)
5. Use: "Meeting_Notes_Q3_Planning.md" ✓

### Example 5: CSV Data File

**File**: `export.csv`

**Current Flow**:
1. Detect: Document
2. Extract metadata: None
3. Content: Read first row → "1,John,Doe,john@example.com"
4. Use: "1_John_Doe.csv" (includes ID)

**Enhanced Flow**:
1. Detect: Document
2. Extract metadata: None
3. Content extraction:
   - Parse CSV with headers
   - Headers: ["id", "first_name", "last_name", "email"]
   - First row: ["1", "John", "Doe", "john@example.com"]
4. Column analysis:
   - "id" → Skip (identifier)
   - "first_name" → Use (semantic)
   - "last_name" → Use (semantic)
   - "email" → Skip (secondary)
5. Combine: "John_Doe"
6. Directory context: /Users/john/Contacts/
7. Final: "Contacts_John_Doe.csv" ✓

### Example 6: Email File

**File**: `message.eml` (not yet implemented)

**Enhanced Flow**:
1. Detect: Document (email format)
2. Parse email headers:
   ```
   From: jane@example.com
   Subject: Q3 Budget Proposal Review
   Date: Mon, 15 Oct 2023 14:30:22 +0000
   ```
3. Extract:
   - Subject: "Q3 Budget Proposal Review"
   - From: "jane@example.com" → "jane"
   - Date: "2023-10-15"
4. Combine: "Q3_Budget_Proposal_Review_jane_2023_10_15"
5. Score: 9.1 (comprehensive, descriptive)
6. Use: "Q3_Budget_Proposal_Review_jane_2023_10_15.eml" ✓

---

## Implementation Guidelines

### Adding New Heuristics

When implementing new heuristics:

1. **Modular design**: Each heuristic as separate function
2. **Scoring integration**: Return scored candidates
3. **Graceful degradation**: Handle missing tools/libraries
4. **Comprehensive testing**: Unit tests with edge cases
5. **Performance consideration**: Profile before/after
6. **Documentation**: Update this file with examples

### Testing Heuristics

Create test cases covering:

- **Happy path**: Ideal input with good metadata
- **Missing metadata**: Fallback to content extraction
- **Garbage input**: OCR noise, corrupted files
- **Edge cases**: Empty files, binary data, special characters
- **Performance**: Large files, batch operations

### Performance Targets

- **Single file**: < 2 seconds (including OCR)
- **Batch (100 files)**: < 3 minutes with caching
- **Memory usage**: < 100MB peak for typical operation
- **Cache hit speedup**: 5-10x faster for re-runs

---

## References

- **Implementation task**: See `sessions/tasks/enhance-naming-heuristics.md`
- **Code modules**:
  - `src/extractor.rs` - Metadata extraction and filtering
  - `src/generator.rs` - Filename sanitization and generation
  - `src/detector.rs` - File type detection
  - `src/pdf_content.rs` - PDF text extraction
  - `src/text_content.rs` - Plain text extraction
  - `src/image_ocr.rs` - Image OCR
  - `src/video_ocr.rs` - Video frame OCR

---

## Advanced Features (Recently Implemented)

### Multi-Frame Video Analysis

**Feature**: Extract and analyze multiple video frames to find the best OCR result.

**Implementation**: Extracts frames at 00:00:01, 00:00:05, and 00:00:10, runs OCR on each, then uses the quality scoring system to select the best result.

**Usage**: `nameback /path/to/videos --multiframe-video`

**Example**:
- Single frame (at 1s) might catch a transition: "Loading..."
- Multi-frame catches better moment (at 5s): "Product Demo Video"

**Performance**: 3x slower than single-frame, but significantly more accurate.

### Series Detection and Numbering

**Feature**: Automatically detect file series and maintain sequence numbering.

**Detects patterns**:
- `IMG_001.jpg`, `IMG_002.jpg`, `IMG_003.jpg`
- `Screenshot(1).png`, `Screenshot(2).png`, `Screenshot(3).png`
- `report-1.pdf`, `report-2.pdf`, `report-3.pdf`
- `vacation 1.jpg`, `vacation 2.jpg`, `vacation 3.jpg`

**Behavior**: When 3+ files match a series pattern, maintains numbering in new names:
```
IMG_001.jpg → beach_sunset_001.jpg
IMG_002.jpg → beach_sunset_002.jpg
IMG_003.jpg → beach_sunset_003.jpg
```

### Format-Specific Handlers

#### Email Files (.eml, .msg)

Extracts: Subject, From, Date
Format: `Subject_from_Sender_YYYY-MM-DD.eml`

Example:
```
message.eml → Weekly_Status_Report_from_John_Smith_2023-10-15.eml
```

#### Web Archives (.html, .mhtml)

Extracts: `<title>` tag, meta description
Removes: Site suffixes (" - Google Search", " | Facebook")

Example:
```
page.html → Important_Documentation_Page.html
```

#### Archive Files (.zip, .tar, .7z, .rar)

Strategy:
1. If single file inside: Use that filename
2. If multiple files with common prefix: Use the prefix
3. Fallback: Use directory context

Example:
```
archive.zip (contains project_proposal.pdf) → project_proposal.zip
```

#### Source Code Files

Extracts docstrings from module-level documentation:

- **Python**: First `"""..."""` or `'''...'''` docstring
- **JavaScript/TypeScript**: `@file` or `@module` JSDoc tags
- **Rust**: `//!` module comments
- **Java**: Javadoc before class
- **C/C++**: Doxygen `/**` comments

Example:
```python
"""
User authentication module.
Handles login, logout, and session management.
"""
# → user_authentication_module.py
```

### Location & Timestamp Enrichment

**GPS Coordinates**: Optionally add location to photo/video filenames.

Usage: `nameback /photos --include-location`

Example:
```
IMG_2847.jpg → sunset_37.77N_122.42W.jpg  (San Francisco)
```

**Formatted Timestamps**: Add human-readable dates as fallback.

Usage: `nameback /photos --include-timestamp`

Example:
```
IMG_20231015_143022.jpg → 2023-10-15_afternoon.jpg
```

---

## Implementation Files

**Advanced features added in**:
- `src/series_detector.rs` - Series pattern detection
- `src/location_timestamp.rs` - GPS and timestamp formatting
- `src/code_docstring.rs` - Source code docstring extraction
- `src/format_handlers/email.rs` - Email metadata extraction
- `src/format_handlers/web.rs` - HTML title extraction
- `src/format_handlers/archive.rs` - Archive content inspection
- `src/video_ocr.rs` - Multi-frame video analysis (extract_video_text_multiframe)

---

**Document Version**: 2.0
**Last Updated**: 2025-10-20
**Status**: All planned enhancements implemented and tested
