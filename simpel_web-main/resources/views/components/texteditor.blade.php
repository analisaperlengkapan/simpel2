<link href="{{ asset('assets/libs/quill/quill.snow.css') }}" rel="stylesheet" type="text/css" />

@php
    $idnya = $id ?? $name;
@endphp

@isset($label)
    <label for="{{ $idnya }}" class="form-label">{{ $label }}</label>
@endisset

<div class="snow-editor" style="height: 300px;">
</div>
<input type="hidden" name="{{ $name }}" id="{{ $idnya }}" value="{{ $value }}" />
<script src="{{ asset('assets/libs/quill/quill.min.js') }}"></script>
<script>
    const snowEditor = document.querySelectorAll(".snow-editor");
    if (snowEditor) {
        Array.from(snowEditor).forEach(function(item) {
            const snowEditorData = {};
            const issnowEditorVal = item.classList.contains("snow-editor");
            if (issnowEditorVal == true) {
                snowEditorData.theme = 'snow',
                    snowEditorData.modules = {
                        'toolbar': [
                            [{
                                'font': []
                            }, {
                                'size': []
                            }],
                            ['bold', 'italic', 'underline', 'strike'],
                            [{
                                'color': []
                            }, {
                                'background': []
                            }],
                            [{
                                'script': 'super'
                            }, {
                                'script': 'sub'
                            }],
                            [{
                                'header': [false, 1, 2, 3, 4, 5, 6]
                            }, 'blockquote', 'code-block'],
                            [{
                                'list': 'ordered'
                            }, {
                                'list': 'bullet'
                            }, {
                                'indent': '-1'
                            }, {
                                'indent': '+1'
                            }],
                            ['direction', {
                                'align': []
                            }],
                            ['link', 'image'],
                            ['clean']
                        ]
                    }
            }
            const quill = new Quill(item, snowEditorData);
            quill.root.innerHTML = `{!! $value !!}`;
            quill.on('text-change', function() {
                const isi = quill.root.innerHTML;
                const hiddenId = `{{ $idnya }}`
                $('#' + hiddenId).val(isi);
            })
        });
    }
</script>
