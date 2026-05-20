# SIMPEL Design System Documentation

## Overview

Arsitektur CSS yang terstruktur dan scalable untuk sistem SIMPEL dengan pendekatan modular dan best practices.

## Structure

```
antarmuka/shared/styles/
├── main.css        # Entry point untuk semua styles
├── variables.css   # CSS Custom Properties & theme variables  
├── base.css        # Reset & base element styles
├── layout.css      # Layout components (header, footer, grid)
├── components.css  # UI components (buttons, cards, forms)
├── utilities.css   # Utility classes (spacing, typography)
└── legacy.css      # Backward compatibility support
```

## Architecture Principles

### 1. **Separation of Concerns**

- **Variables**: Design tokens & theme configuration
- **Base**: Element resets & typography  
- **Layout**: Structural components
- **Components**: Reusable UI patterns
- **Utilities**: Single-purpose classes
- **Legacy**: Backward compatibility

### 2. **Dependency Order**

```css
@import 'variables.css';    /* First - defines tokens */
@import 'base.css';        /* Second - element defaults */
@import 'layout.css';      /* Third - page structure */
@import 'components.css';  /* Fourth - UI components */
@import 'utilities.css';   /* Last - override classes */
```

### 3. **Naming Convention**

#### CSS Custom Properties (Variables)

```css
--color-primary-500     /* Color palette */
--spacing-4            /* Spacing scale */
--font-size-lg         /* Typography scale */
--border-radius-md     /* Border radius scale */
```

#### Layout Classes

```css
.app-header           /* Application-level layouts */
.content-section      /* Content structure */
.page-header          /* Page-level components */
```

#### Component Classes

```css
.btn-primary          /* UI components */
.card-header          /* Component parts */
.form-input          /* Form elements */
```

#### Utility Classes

```css
.p-4                 /* Padding utilities */
.text-center         /* Text utilities */
.grid-cols-3         /* Grid utilities */
```

## Usage Guide

### 1. **Standard Implementation**

```html
<!-- Tailwind CSS for rapid development -->
<link href="https://cdn.jsdelivr.net/npm/tailwindcss@3.4.0/dist/tailwind.min.css" rel="stylesheet">

<!-- SIMPEL Design System -->
<link rel="stylesheet" href="../shared/styles/main.css">

<!-- Legacy support for existing components -->
<link rel="stylesheet" href="../shared/styles/legacy.css">
```

### 2. **Modern Development (Recommended)**

Use Tailwind CSS for new components:

```html
<div class="bg-white rounded-lg shadow-md p-6">
  <h2 class="text-xl font-semibold text-gray-800 mb-4">Modern Component</h2>
  <p class="text-gray-600">Using Tailwind utilities</p>
</div>
```

### 3. **Legacy Support**

Existing components continue working:

```html
<div class="card">
  <div class="card-header">
    <h2 class="card-title">Legacy Component</h2>
  </div>
  <p>Existing styling preserved</p>
</div>
```

## Migration Strategy

### Phase 1: **Foundation** ✅ COMPLETED

- [x] Restructure CSS architecture
- [x] Create design system files
- [x] Add Tailwind CSS integration  
- [x] Maintain backward compatibility

### Phase 2: **Gradual Migration** 🔄 IN PROGRESS

- [ ] Migrate BADIKLAT components to new system
- [ ] Update shared components in Rust
- [ ] Convert legacy classes to utilities
- [ ] Remove unused CSS rules

### Phase 3: **Optimization** 📋 PLANNED

- [ ] Remove legacy.css dependency
- [ ] Implement CSS purging for production
- [ ] Add component documentation
- [ ] Performance optimization

## Benefits

### ✅ **Improved Structure**

- Clear separation of concerns
- Scalable architecture
- Easy maintenance

### ✅ **Reduced Duplication**

- Single source of truth
- Shared design tokens
- Consistent theming

### ✅ **Better Developer Experience**

- Tailwind CSS integration
- Utility-first approach
- Component-based thinking

### ✅ **Backward Compatibility**

- Existing components still work
- Gradual migration path
- No breaking changes

## File Size Analysis

### Before Refactoring

```
perlengkapan.css (duplicate): 539 lines × 8 files = 4,312 lines
Total CSS: ~3,243 lines
```

### After Refactoring

```
main.css (entry):      10 lines
variables.css:         374 lines  
base.css:             576 lines
layout.css:           180 lines (new)
components.css:       506 lines
utilities.css:        120 lines (new)  
legacy.css:           150 lines (new)
Total CSS: ~1,916 lines (40% reduction)
```

## Best Practices

### 1. **Use Design Tokens**

```css
/* Good */
color: var(--color-primary-600);

/* Avoid */
color: #2563eb;
```

### 2. **Prefer Utilities for New Code**

```html
<!-- Good -->
<div class="p-4 bg-white rounded-lg shadow">

<!-- Legacy (acceptable for existing code) -->
<div class="card">
```

### 3. **Component-Specific Styles**

```css
/* Keep component-specific styles in individual CSS files */
/* badiklat.css, datun.css, etc. */
```

### 4. **Responsive Design**

```css
/* Use mobile-first approach */
.component {
  /* Mobile styles */
}

@media (min-width: 768px) {
  .component {
    /* Tablet+ styles */
  }
}
```

## Migration Guide

### For Existing Components

1. Keep current CSS references working
2. Add Tailwind classes alongside existing classes
3. Gradually replace custom CSS with utilities
4. Remove unused styles when confident

### For New Components

1. Use Tailwind CSS utilities primarily
2. Add custom CSS only when necessary
3. Use design system variables
4. Follow component naming conventions

This refactored system provides a solid foundation for scaling SIMPEL's frontend architecture while maintaining full backward compatibility.
