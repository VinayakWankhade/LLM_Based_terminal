# MCQ Generation Service (OCR + NLP)

This project builds a backend service that converts PDFs or images of text into high-quality multiple-choice questions (MCQs).

Core pipeline: OCR/Text Extraction → Preprocess → Concept Mining → Question Generation → Answer Validation → Quality Balancing → Export (JSON/CSV/PDF).

Features
- Accepts PDFs and images (handwritten/printed; OCR via Tesseract)
- Cleans and segments text into meaningful chunks
- Generates MCQs with one correct answer and plausible distractors
- Heuristic QG with optional Transformers (e.g., FLAN-T5) when enabled
- Quality checks: deduplicate, validate, and balance difficulties (30/50/20)
- Exports to JSON, CSV, or PDF
- FastAPI endpoints for easy integration

Tech Stack
- Python, FastAPI
- pdfplumber (PDF text extraction)
- pytesseract + Pillow (OCR for images)
- transformers (optional, for improved QG/QA)
- reportlab (PDF export)

Prerequisites
- Python 3.10 or newer recommended
- Tesseract OCR installed (Windows installer typically at: C:\\Program Files\\Tesseract-OCR)
  - If not in PATH, set environment variable TESSERACT_CMD to the full path of tesseract.exe

Quickstart (Windows, PowerShell)
1) Create and activate a virtual environment
   python -m venv .venv
   .\.venv\Scripts\Activate.ps1

2) Install dependencies
   pip install -r requirements.txt

   Optional model data (only if you want WordNet-based distractors):
   python -c "import nltk; nltk.download('wordnet'); nltk.download('omw-1.4')"

   Note: If you plan to enable transformers-based QG/QA, install transformers and the correct PyTorch build for your system.
   pip install transformers
   # For PyTorch, follow https://pytorch.org/get-started/locally/ for the right command.

3) Run the API server
   uvicorn mcqgen.backend.app.main:app --reload

4) Test the endpoints
- Extract text from files (PDF or image):
  POST http://localhost:8000/api/extract-text (multipart/form-data, field name: files)

- Generate MCQs from raw text:
  POST http://localhost:8000/api/generate (application/json)
  {
    "text": "<your extracted or raw academic text>",
    "num_questions": 10,
    "use_transformers": false,
    "target_mix": {"easy": 0.3, "medium": 0.5, "hard": 0.2}
  }

- Export MCQs to CSV/PDF:
  POST http://localhost:8000/api/export (application/json)
  {
    "mcqs": [ ... ],
    "format": "csv"  // or "pdf" or "json"
  }

Environment Configuration
- TESSERACT_CMD: Full path to tesseract.exe if not in PATH (e.g., C:\\Program Files\\Tesseract-OCR\\tesseract.exe)

Notes on Models
- By default, the service uses heuristic question generation to avoid heavy downloads.
- If use_transformers is true, the service will attempt to load a lightweight model like FLAN-T5-small for QG (if installed). This may download weights on first run.
- For answer validation with QA models, you can configure the code to use a QA pipeline (e.g., a SQuAD-tuned model). Otherwise, basic substring validation is used.

Limitations and Next Steps
- The heuristic QG is domain-agnostic and may be less fluent than LLM-based generation.
- Consider fine-tuning or plugging in larger models for better quality when resources allow.
- A simple React front-end can be added later for file upload and result viewing.

Project Layout
mcqgen/
  backend/
    app/           FastAPI app and schemas
    services/      OCR, PDF parsing, preprocess, QG, validation, export


