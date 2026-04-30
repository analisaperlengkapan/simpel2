@isset($label)
    <label for="{{ $id ?? $name }}" class="form-label">{{ $label }}</label>
@endisset
@php
    $time = $withTime ?? false ? 'data-enable-time' : '';
    $disabled = $withDisabled ?? false ? 'disabled' : '';
@endphp
<div class="form-icon right">
    <input data-provider="flatpickr" class="form-control {{ $className ?? '' }}" name="{{ $name }}"
        id="{{ $id ?? $name }}" placeholder="{{ $placeholder ?? 'Pilih Tanggal' }}" value="{{ $value ?? '' }}"
        data-column="{{ $index ?? '' }}" {{ $time }} {{ $disabled }}  />
    <i class="ri-calendar-2-fill"></i>
</div>
