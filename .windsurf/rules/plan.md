---
trigger: manual
---

{
  "version": "2.0.0",
  "rules": [
    {
      "id": "deep-planning",
      "name": "Deep Planning Workflow",
      "description": "Mode perencanaan mendalam dan terstruktur untuk project development: Requirements → Design → Tasks → Implementation",
      "icon": "📋",
      "mode": "structured",
      "temperature": 0.3,
      "capabilities": {
        "multiTurn": true,
        "fileOperations": true,
        "codeGeneration": true
      },
      "tools": [
        "workspace",
        "files",
        "codebase",
        "think",
        "sequentialthinking"
      ],
      "workflow": {
        "entrypoint": "start",
        "steps": {
          "start": {
            "prompt": [
              "📋 Selamat datang di *Deep Planning Workflow*!",
              "Mari kita mulai dengan memahami project Anda.",
              "",
              "**Silakan jelaskan:**",
              "- Tujuan project / masalah yang ingin diselesaikan",
              "- Target pengguna",
              "- Fitur utama yang diinginkan",
              "",
              "Saya akan memandu melalui 4 tahap utama:",
              "1️⃣ Requirements → 2️⃣ Design → 3️⃣ Tasks → 4️⃣ Implementation",
              "",
              "Setiap dokumen akan terstruktur dengan penomoran sistematis (1, 1.1, 1.1.1) dan *traceability* antar tahap dijaga melalui referensi lintas dokumen."
            ],
            "next": "requirements"
          },

          "requirements": {
            "instructions": [
              "📋 **TAHAP 1: REQUIREMENTS GATHERING**",
              "",
              "Gunakan urutan tool berikut:",
              "1. `@codebase` → Analisis struktur & konteks project",
              "2. `@think` → Reasoning kebutuhan & cakupan",
              "3. `@sequentialthinking` → Breakdown kebutuhan sistematis",
              "",
              "Langkah-langkah:",
              "- Analisis project, arsitektur, dan dependensi existing",
              "- Identifikasi kebutuhan bisnis, fungsional, dan non-fungsional",
              "- Buat file `requirements.md` dengan struktur berpenomoran:",
              "",
              "```markdown",
              "# Requirements Document",
              "## 1. Project Overview",
              "### 1.1 Project Goals",
              "### 1.2 Target Users",
              "### 1.3 Scope",
              "",
              "## 2. Business Requirements",
              "### 2.1 [Kebutuhan bisnis 1]",
              "",
              "## 3. Functional Requirements",
              "### 3.1 [Fitur utama 1]",
              "#### 3.1.1 [Sub-fitur]",
              "",
              "## 4. Non-Functional Requirements",
              "### 4.1 Performance",
              "### 4.2 Security",
              "",
              "## 5. Constraints & Assumptions",
              "### 5.1 Technical Constraints",
              "### 5.3 Assumptions",
              "",
              "## 6. Success Criteria",
              "### 6.1 [Kriteria 1]",
              "```",
              "",
              "✅ **Aturan:**",
              "- Semua poin wajib bernomor (1, 1.1, 1.1.1)",
              "- Hindari bullet list biasa",
              "- Setelah selesai, tampilkan checkpoint konfirmasi"
            ],
            "confirm": {
              "template": [
                "---",
                "⏸️ CHECKPOINT - REQUIREMENTS",
                "",
                "File dibuat: `requirements.md` berisi {{mainPoints}} poin utama dan {{subPoints}} sub-poin.",
                "",
                "Opsi:",
                "1. ✅ Approve - lanjut ke Design",
                "2. 🔧 Revisi - sebutkan nomor poin yang perlu diubah",
                "3. 💬 Diskusi - klarifikasi lebih lanjut",
                "---"
              ]
            },
            "next": "design"
          },

          "design": {
            "instructions": [
              "🎨 **TAHAP 2: DESIGN DOCUMENTATION**",
              "",
              "Mulai hanya setelah *requirements* disetujui.",
              "Gunakan tools:",
              "- `@codebase` → Pelajari arsitektur existing",
              "- `@think` → Evaluasi trade-off design",
              "- `@sequentialthinking` → Pecah tiap komponen desain",
              "",
              "Hasilkan file `design.md` dengan format:",
              "```markdown",
              "# Design Document",
              "## 1. Architecture Overview",
              "**Refs:** [Req 1.1, 3.1]",
              "",
              "## 2. Module Design",
              "### 2.1 [Module Name]",
              "**Refs:** [Req 3.1.1, 3.1.2]",
              "#### 2.1.1 Responsibilities",
              "#### 2.1.2 Interfaces",
              "",
              "## 3. Data Models",
              "**Refs:** [Req 3.1]",
              "### 3.1 Database Schema",
              "### 3.2 Relationships",
              "",
              "## 4. API Design",
              "**Refs:** [Req 3.2, 4.2]",
              "### 4.1 Endpoints",
              "### 4.2 Auth & Authorization",
              "",
              "## 5. Technology Stack",
              "**Refs:** [Req 5.1]",
              "### 5.1 Frontend",
              "### 5.2 Backend",
              "### 5.3 Infrastructure",
              "",
              "## 6. Security Design",
              "**Refs:** [Req 4.2]",
              "### 6.1 Encryption",
              "### 6.2 Secure Config",
              "",
              "## 7. Design Decisions",
              "### 7.1 [Decision]",
              "**Rationale:** ...",
              "```",
              "",
              "✅ **Aturan penting:**",
              "- Setiap section wajib ada `Refs: [Req X.X]`",
              "- Referensi harus spesifik (bukan hanya [Req 3])",
              "- Pastikan traceability antara requirements → design"
            ],
            "confirm": {
              "template": [
                "---",
                "⏸️ CHECKPOINT - DESIGN",
                "",
                "File dibuat: `design.md` dengan {{modules}} modul & {{refs}} referensi ke requirements.",
                "",
                "Opsi:",
                "1. ✅ Approve - lanjut ke Tasks",
                "2. 🔧 Revisi - sebutkan nomor modul yang perlu diubah",
                "3. 💬 Diskusi - klarifikasi keputusan desain",
                "---"
              ]
            },
            "next": "tasks"
          },

          "tasks": {
            "instructions": [
              "✅ **TAHAP 3: TASK BREAKDOWN**",
              "",
              "Mulai hanya setelah design disetujui.",
              "Buat `tasks.md` dengan checklist, referensi, dan estimasi:",
              "",
              "```markdown",
              "# Tasks Document",
              "## 1. Setup & Infrastructure",
              "### 1.1 Project Initialization",
              "**Requirements:** [Req 5.1]",
              "**Design:** [Design 5.1, 5.2]",
              "- [ ] 1.1.1 Setup repository",
              "- [ ] 1.1.2 Configure CI/CD",
              "**Priority:** High | **Est:** 2h",
              "",
              "## 2. Backend Development",
              "### 2.1 Authentication Module",
              "**Requirements:** [Req 3.1.1, 4.2]",
              "**Design:** [Design 2.1, 6.1]",
              "- [ ] 2.1.1 Implement user registration",
              "- [ ] 2.1.2 Implement login/logout",
              "**Priority:** High | **Est:** 6h",
              "**Acceptance Criteria:**",
              "  - JWT valid & password hashed",
              "",
              "## 3. Frontend Development",
              "### 3.1 UI Components",
              "**Requirements:** [Req 3.1, 4.4]",
              "**Design:** [Design 2.1, 5.1]",
              "- [ ] 3.1.1 Create components",
              "- [ ] 3.1.2 Setup routing",
              "",
              "---",
              "## Progress Summary",
              "- Total Tasks: [X]",
              "- Completed: 0/[X]",
              "- Pending: [X]",
              "```",
              "",
              "📌 **Rule Format:**",
              "- Numbering konsisten (1.1, 1.1.1)",
              "- Checkbox hanya untuk sub-tasks",
              "- Requirements & Design wajib dipisah",
              "- Priority dan estimasi wajib ada",
              "- Acceptance Criteria untuk task kompleks",
              "- Dependencies dicantumkan jika ada"
            ],
            "confirm": {
              "template": [
                "---",
                "⏸️ CHECKPOINT - TASKS",
                "",
                "File dibuat: `tasks.md` dengan {{totalTasks}} total task dan {{groups}} kelompok utama.",
                "",
                "Opsi:",
                "1. ✅ Approve - lanjut ke Implementation",
                "2. 🔧 Revisi - sebutkan nomor task yang perlu diubah",
                "3. 💬 Diskusi - klarifikasi breakdown",
                "---"
              ]
            },
            "next": "implementation"
          },

          "implementation": {
            "instructions": [
              "🚀 **TAHAP 4: IMPLEMENTATION**",
              "",
              "Gunakan tool `@codebase` untuk cari contoh dan `@think` untuk reasoning tiap task.",
              "Implementasikan task **satu per satu** sesuai urutan prioritas dan dependencies.",
              "",
              "Untuk setiap task:",
              "- Tampilkan status: `🚀 Mengerjakan Task [Nomor]: [Nama]`",
              "- Tampilkan referensi: `📋 [Requirements] | 🎨 [Design]`",
              "- Tambahkan komentar & dokumentasi kode",
              "- Sertakan unit tests jika perlu",
              "",
              "Setelah selesai:",
              "- Update `tasks.md`: ubah `- [ ]` jadi `- [x]`",
              "- Update Progress Summary",
              "- Tanyakan konfirmasi: *'Task [Nomor] selesai. Review atau lanjut?'*"
            ],
            "confirm": {
              "template": [
                "---",
                "✅ IMPLEMENTATION COMPLETE",
                "",
                "Semua task selesai. Progress Summary diperbarui.",
                "",
                "Opsi:",
                "1. 🎉 Selesai - Tutup workflow",
                "2. 🔁 Ulangi tahap tertentu untuk revisi",
                "---"
              ]
            }
          }
        }
      },

      "validation": {
        "rules": [
          "- Setiap file wajib bernomor hierarkis (1, 1.1, 1.1.1)",
          "- Semua `Refs`, `Requirements`, dan `Design` harus valid dan ada",
          "- Tidak boleh ada [TODO] atau placeholder",
          "- Jumlah task di Progress Summary harus sesuai dengan jumlah checklist",
          "- Tidak boleh skip tahap tanpa konfirmasi user"
        ]
      },

      "communication": {
        "style": "clear-structured",
        "emojis": {
          "requirements": "📋",
          "design": "🎨",
          "tasks": "✅",
          "implementation": "🚀"
        },
        "confirmationFormat": [
          "---",
          "⏸️ CHECKPOINT - {{phaseName}}",
          "{{summary}}",
          "",
          "Opsi:",
          "1. ✅ Approve - lanjut",
          "2. 🔧 Revisi - sebutkan nomor poin",
          "3. 💬 Diskusi - klarifikasi",
          "---"
        ]
      }
    }
  ]
}
