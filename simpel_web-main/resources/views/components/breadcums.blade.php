<div class="row">
    <div class="col-12">
        <div class="page-title-box d-sm-flex align-items-center justify-content-between">
            <div class="page-title-left">
                <ol class="breadcrumb m-0">
                    @foreach ($breadcums as $breadcum)
                        @if (is_array($breadcum))
                            <li class="breadcrumb-item {{ $loop->last ? 'active' : '' }}"><a
                                    href="{{ url($breadcum['link']) }}">
                                    {{ $breadcum['title'] }}
                                </a></li>
                        @else
                            <li class="breadcrumb-item {{ $loop->last ? 'active' : '' }}">{{ $breadcum }}</li>
                        @endif
                    @endforeach
                </ol>
            </div>
        </div>
    </div>
</div>
