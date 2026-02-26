@extends('layout.main')
@section('content')
    @include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-xxl-9">
            <form action="{{ $controller }}" method="POST" class="ajaxForm">
                <div class="card mt-xxl-n5">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">{{ $isNew ? 'Tambah' : 'Edit' }} Role Group Privileges</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body p-4">
                        @csrf
                        @if (!$isNew)
                            <input type="hidden" id="id" name="id" value="{{ $model['id'] }}">
                        @endif
                        <div class="row">
                            <div class="col-lg-3">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama</label>
                                    <input class="form-control" id="name" name="name" placeholder="Nama Role"
                                        required value="{{ $model['name'] ?? '' }}">
                                </div>
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">Deskripsi</label>
                                    <input type="text" class="form-control" id="description" name="description" required
                                        placeholder="Deskripsi Role" value="{{ $model['description'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="card mt-xxl-n5">
                    <div class="card-header">
                        <div class="d-flex align-items-center">
                            <div class="flex-grow-1">
                                <h5 class="card-title mb-0">Akses Menu</h5>
                            </div>
                        </div>
                    </div>
                    <div class="card-body p-4">
                        <div class="col">
                            @foreach ($menus2 as $level1)
                                @php($checked = empty($level1['menu_id']) ? '' : 'checked')
                                <div class="custom-control custom-checkbox">
                                    <input type="checkbox" class="custom-control-input" id="menu-{{ $level1['id'] }}"
                                        name="menu_id[]" {{ $checked }} data-level="1" value="{{ $level1['id'] }}">
                                    <label class="custom-control-label"
                                        for="menu-{{ $level1['id'] }}">{{ $level1['name'] }}</label>

                                    @if (isset($level1['childs']))
                                        <ul>
                                            @foreach ($level1['childs'] as $level2)
                                                @php($checked2 = empty($level2['menu_id']) ? '' : 'checked')
                                                @php($parentId = $level2['parent'])
                                                <div class="custom-control custom-checkbox">
                                                    <input type="checkbox" class="custom-control-input"
                                                        id="menu-{{ $level2['id'] }}" name="menu_id[]" {{ $checked2 }}
                                                        data-parent="{{ $parentId }}" data-level="2"
                                                        value="{{ $level2['id'] }}">
                                                    <label class="custom-control-label"
                                                        for="menu-{{ $level2['id'] }}">{{ $level2['name'] }}</label>
                                                </div>

                                                @if (isset($level2['childs']))
                                                    <ul>
                                                        @foreach ($level2['childs'] as $level3)
                                                            @php($checked3 = empty($level3['menu_id']) ? '' : 'checked')
                                                            @php($parentId = $level3['parent'])
                                                            <div class="custom-control custom-checkbox">
                                                                <input type="checkbox" class="custom-control-input"
                                                                    id="menu-{{ $level3['id'] }}" name="menu_id[]"
                                                                    {{ $checked3 }} data-parent="{{ $parentId }}"
                                                                    data-level="3" value="{{ $level3['id'] }}">
                                                                <label class="custom-control-label"
                                                                    for="menu-{{ $level3['id'] }}">{{ $level3['name'] }}</label>

                                                            </div>
                                                            @if (isset($level3['childs']))
                                                                <ul>
                                                                    @foreach ($level3['childs'] as $level4)
                                                                        @php($checked4 = empty($level4['menu_id']) ? '' : 'checked')
                                                                        @php($parentId = $level4['parent'])
                                                                        <div class="custom-control custom-checkbox">
                                                                            <input type="checkbox"
                                                                                class="custom-control-input"
                                                                                id="menu-{{ $level4['id'] }}"
                                                                                name="menu_id[]" {{ $checked4 }}
                                                                                data-parent="{{ $parentId }}"
                                                                                data-level="4" value="{{ $level4['id'] }}">

                                                                            <label class="custom-control-label"
                                                                                for="menu-{{ $level4['id'] }}">{{ $level4['name'] }}</label>
                                                                        </div>
                                                                    @endforeach
                                                                </ul>
                                                            @endif
                                                        @endforeach
                                                    </ul>
                                                @endif
                                            @endforeach
                                        </ul>
                                    @endif
                                </div>
                            @endforeach
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content">
                                    <a href="{{ $controller }}" class="btn btn-outline-primary">Kembali</a>
                                    <button type="submit" class="btn btn-primary">
                                        {{ $isNew ? 'Simpan' : 'Ubah' }}
                                    </button>

                                </div>
                            </div>
                        </div>

                    </div>
                </div>
            </form>


        </div>
    </div>
@endsection



@section('js')
    <script>
        $(function() {

        })
    </script>
@endsection
