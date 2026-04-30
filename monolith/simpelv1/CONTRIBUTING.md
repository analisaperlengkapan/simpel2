# Panduan Kontribusi

Terima kasih telah tertarik untuk berkontribusi pada **Simpel Web** - Sistem Manajemen Aset dan Pengadaan Kejaksaan Republik Indonesia!

## 📋 Daftar Isi

- [Cara Kontribusi](#cara-kontribusi)
- [Standar Kode](#standar-kode)
- [Git Workflow](#git-workflow)
- [Laporan Bug](#laporan-bug)
- [Saran Fitur](#saran-fitur)
- [Pull Request](#pull-request)
- [Code Review](#code-review)
- [Testing](#testing)
- [Dokumentasi](#dokumentasi)
- [Keamanan](#keamanan)

## 🤝 Cara Kontribusi

### Langkah-langkah Kontribusi

1. **Fork repository ini**
   - Klik tombol "Fork" di GitHub/GitLab
   - Clone repository fork Anda ke local

2. **Setup development environment**
   ```bash
   git clone <your-fork-url>
   cd simpel_web
   composer install
   npm install
   cp .env.example .env
   php artisan key:generate
   ```

3. **Buat branch baru**
   ```bash
   git checkout -b feature/nama-fiturnya
   # atau
   git checkout -b bugfix/nama-bug
   # atau
   git checkout -b hotfix/nama-hotfix
   ```

4. **Lakukan perubahan**
   - Tulis kode sesuai standar
   - Tambahkan test jika diperlukan
   - Update dokumentasi

5. **Commit perubahan**
   ```bash
   git add .
   git commit -m "feat: tambah fitur manajemen aset"
   ```

6. **Push ke repository fork**
   ```bash
   git push origin feature/nama-fiturnya
   ```

7. **Buat Pull Request/Merge Request**
   - Buka repository fork Anda
   - Klik "New Pull Request" atau "Create Merge Request"
   - Isi template yang disediakan

## 📝 Standar Kode

### PHP/Laravel Standards

- Ikuti **PSR-12** coding standards
- Gunakan **Laravel Pint** untuk formatting
- Ikuti **Laravel naming conventions**
- Gunakan **type hints** dan **return types**
- Tulis **PHPDoc** untuk method public

```php
/**
 * Get asset by ID
 *
 * @param int $id
 * @return Asset|null
 */
public function getAssetById(int $id): ?Asset
{
    return Asset::find($id);
}
```

### JavaScript/Vue Standards

- Gunakan **ES6+** syntax
- Ikuti **Vue.js style guide**
- Gunakan **Prettier** untuk formatting
- Tulis **JSDoc** untuk functions

### Database Standards

- Gunakan **Laravel migrations**
- Ikuti **naming conventions** untuk tabel dan kolom
- Tulis **database seeders** untuk data testing
- Gunakan **foreign key constraints**

### Testing Standards

- Tulis **unit tests** untuk business logic
- Tulis **feature tests** untuk controllers
- Gunakan **database factories** untuk test data
- Target **minimum 80% code coverage**

## 🔄 Git Workflow

### Branch Naming Convention

```
feature/    - Fitur baru
bugfix/     - Perbaikan bug
hotfix/     - Perbaikan urgent
docs/       - Dokumentasi
refactor/   - Refactoring kode
test/       - Penambahan test
```

### Commit Message Convention

Gunakan **Conventional Commits**:

```
feat:     - Fitur baru
fix:      - Perbaikan bug
docs:     - Dokumentasi
style:    - Formatting, missing semicolons, etc
refactor: - Refactoring kode
test:     - Menambah test
chore:    - Maintenance tasks
```

Contoh:
```bash
git commit -m "feat: tambah QR code generator untuk aset"
git commit -m "fix: perbaiki bug pada export Excel"
git commit -m "docs: update README dengan instruksi instalasi"
```

## 🐛 Laporan Bug

### Template Bug Report

```markdown
**Deskripsi Bug**
Penjelasan singkat tentang bug yang ditemukan.

**Langkah Reproduksi**
1. Buka halaman '...'
2. Klik pada '...'
3. Scroll ke '...'
4. Lihat error

**Perilaku yang Diharapkan**
Penjelasan tentang apa yang seharusnya terjadi.

**Screenshot**
Jika memungkinkan, tambahkan screenshot.

**Environment**
- OS: [e.g. Ubuntu 20.04]
- Browser: [e.g. Chrome 91]
- PHP Version: [e.g. 8.1]
- Laravel Version: [e.g. 10.48.29]

**Informasi Tambahan**
Konteks lain tentang masalah ini.
```

### Kriteria Bug Report yang Baik

- ✅ **Reproducible**: Bug dapat direproduksi secara konsisten
- ✅ **Specific**: Deskripsi yang jelas dan spesifik
- ✅ **Complete**: Semua informasi yang diperlukan
- ✅ **Isolated**: Bug tidak terkait dengan masalah lain

## 💡 Saran Fitur

### Template Feature Request

```markdown
**Ringkasan**
Penjelasan singkat tentang fitur yang diusulkan.

**Masalah yang Dipecahkan**
Penjelasan tentang masalah yang akan dipecahkan oleh fitur ini.

**Solusi yang Diusulkan**
Deskripsi tentang bagaimana fitur akan bekerja.

**Alternatif yang Dipertimbangkan**
Solusi lain yang telah dipertimbangkan.

**Informasi Tambahan**
Screenshot, mockup, atau referensi lain.
```

### Kriteria Feature Request yang Baik

- ✅ **Clear Value**: Manfaat yang jelas untuk pengguna
- ✅ **Feasible**: Teknis dapat diimplementasikan
- ✅ **Aligned**: Sesuai dengan visi proyek
- ✅ **Detailed**: Deskripsi yang lengkap dan detail

## 🔀 Pull Request

### Template Pull Request

```markdown
**Deskripsi**
Penjelasan singkat tentang perubahan yang dilakukan.

**Tipe Perubahan**
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

**Testing**
- [ ] Unit tests ditambahkan/diperbarui
- [ ] Manual testing dilakukan
- [ ] All tests passing

**Checklist**
- [ ] Kode mengikuti standar coding
- [ ] Self-review kode dilakukan
- [ ] Dokumentasi diperbarui
- [ ] Changelog diperbarui

**Screenshots**
Jika ada perubahan UI, tambahkan screenshot.

**Related Issues**
Closes #123
```

### Kriteria PR yang Baik

- ✅ **Single Purpose**: Satu PR untuk satu perubahan
- ✅ **Well Tested**: Test coverage yang memadai
- ✅ **Documented**: Dokumentasi yang diperbarui
- ✅ **Follows Standards**: Mengikuti standar coding

## 👀 Code Review

### Guidelines untuk Reviewer

- **Be Respectful**: Berikan feedback yang konstruktif
- **Be Specific**: Berikan contoh konkret
- **Be Timely**: Review dalam waktu yang wajar
- **Be Thorough**: Periksa keamanan dan performa

### Checklist Review

- [ ] Kode mengikuti standar
- [ ] Test coverage memadai
- [ ] Dokumentasi diperbarui
- [ ] Tidak ada security issues
- [ ] Performa tidak menurun
- [ ] Backward compatibility terjaga

## 🧪 Testing

### Unit Tests
```bash
# Jalankan semua unit tests
php artisan test --testsuite=Unit

# Jalankan test spesifik
php artisan test --filter=AssetTest

# Jalankan test dengan coverage
php artisan test --coverage
```

### Feature Tests
```bash
# Jalankan feature tests
php artisan test --testsuite=Feature

# Jalankan test untuk controller
php artisan test --filter=AssetControllerTest
```

### Browser Tests
```bash
# Jalankan browser tests (jika ada)
php artisan dusk
```

## 📚 Dokumentasi

### Guidelines Dokumentasi

- **Clear**: Jelas dan mudah dipahami
- **Complete**: Lengkap dan tidak ada yang terlewat
- **Current**: Selalu up-to-date
- **Consistent**: Format yang konsisten

### File Dokumentasi

- `README.md` - Dokumentasi utama
- `CHANGELOG.md` - Catatan perubahan
- `CONTRIBUTING.md` - Panduan kontribusi
- `API.md` - Dokumentasi API (jika ada)
- `DEPLOYMENT.md` - Panduan deployment

## 🔒 Keamanan

### Security Guidelines

- **Never commit secrets**: Jangan commit file .env atau credentials
- **Validate input**: Selalu validasi input user
- **Use prepared statements**: Gunakan Eloquent ORM
- **Follow OWASP guidelines**: Ikuti best practices keamanan
- **Report vulnerabilities**: Laporkan security issues secara private

### Reporting Security Issues

Jika Anda menemukan security vulnerability, **JANGAN** buat issue publik. 
Kirim email ke: `security@kejaksaan.go.id`

## 🏆 Recognition

### Contributors Hall of Fame

Kontributor yang signifikan akan ditambahkan ke:
- README.md contributors section
- CHANGELOG.md contributors
- Project documentation

### Contribution Levels

- **Bronze**: 1-5 contributions
- **Silver**: 6-15 contributions  
- **Gold**: 16+ contributions
- **Platinum**: Major features atau architectural changes

## 📞 Getting Help

### Resources

- **Documentation**: [Laravel Docs](https://laravel.com/docs)
- **Community**: [Laravel Forum](https://laravel.io/forum)
- **Issues**: GitHub/GitLab Issues
- **Discussions**: GitHub/GitLab Discussions

### Contact

- **Email**: `dev@kejaksaan.go.id`
- **Slack**: `#simpel-web-dev`
- **Telegram**: `@simpelweb_dev`

---

**Terima kasih telah berkontribusi pada Simpel Web! 🚀**

*Mari kita bangun sistem manajemen aset dan pengadaan yang lebih baik bersama-sama.*