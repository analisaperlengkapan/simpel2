<div class="app-menu navbar-menu">
    <!-- LOGO -->
    <div class="navbar-brand-box">
        <!-- Dark Logo-->
        <a href="{{ url('') }}" class="logo logo-dark">
            <span class="logo-sm">
                <img src={{ url($logo_kecil ?? '/assets/images/logo-sm.png') }} alt="" height="22">
            </span>
            <span class="logo-lg">
                <img src={{ url($logo ?? '/assets/images/logo-light.png') }} alt="" width="100%">
            </span>
        </a>
        <!-- Light Logo-->
        <a href="{{ url('') }}" class="logo logo-light">
            <span class="logo-sm">
                <img src={{ url($logo_kecil ?? '/assets/images/logo-sm.png') }} alt="" height="22">
            </span>
            <span class="logo-lg">
                <img src={{ url($logo_dark ?? '/assets/images/logo-light.png') }} alt="" width="100%">
            </span>
        </a>
        <button type="button" class="btn btn-sm p-0 fs-20 header-item float-end btn-vertical-sm-hover"
            id="vertical-hover">
            <i class="ri-record-circle-line"></i>
        </button>
    </div>

    <div id="scrollbar">
        <div class="container-fluid">

            <div id="two-column-menu">
            </div>
            <ul class="navbar-nav" id="navbar-nav">
                @foreach ($menus as $lv1)
                    @if (isset($lv1['childs']))
                        <li class="nav-item">
                            <a class="nav-link menu-link" href="#sidebar{{ $lv1['id'] }}" data-bs-toggle="collapse"
                                role="button" aria-expanded="false" aria-controls="sidebar{{ $lv1['id'] }}">
                                <i class="ri-{{ $lv1['icon'] }}"></i> <span>{{ $lv1['name'] }}</span>
                            </a>
                            <div class="collapse menu-dropdown" id="sidebar{{ $lv1['id'] }}">
                                <ul class="nav nav-sm flex-column">
                                    @foreach ($lv1['childs'] as $lv2)
                                        @if (isset($lv2['childs']))
                                            <li class="nav-item ">
                                                <a href="#sidebar{{ $lv2['id'] }}" class="nav-link"
                                                    data-bs-toggle="collapse" role="button" aria-expanded="false"
                                                    aria-controls="sidebar{{ $lv2['id'] }}">
                                                    {{ $lv2['name'] }}
                                                </a>
                                                <div class="collapse menu-dropdown" id="sidebar{{ $lv2['id'] }}">
                                                    <ul class="nav nav-sm flex-column">
                                                        @foreach ($lv2['childs'] as $lv3)
                                                            @if (isset($lv3['childs']))
                                                                <li class="nav-item">
                                                                    <a href="#sidebar{{ $lv3['id'] }}"
                                                                        class="nav-link" data-bs-toggle="collapse"
                                                                        role="button" aria-expanded="false"
                                                                        aria-controls="sidebar{{ $lv3['id'] }}">
                                                                        {{ $lv3['name'] }}
                                                                    </a>
                                                                    <div class="collapse menu-dropdown"
                                                                        id="sidebar{{ $lv3['id'] }}">
                                                                        <ul class="nav nav-sm flex-column">
                                                                            @foreach ($lv3['childs'] as $lv4)
                                                                                <li class="nav-item">
                                                                                    <a href="{{ URL::to($lv4['route']) }}"
                                                                                        data-active="{{ $lv4['id'] }}"
                                                                                        class="nav-link">
                                                                                        {{ $lv4['name'] }}</a>
                                                                                </li>
                                                                            @endforeach
                                                                        </ul>
                                                                    </div>
                                                                </li>
                                                            @else
                                                                <li class="nav-item">
                                                                    <a href="{{ URL::to($lv3['route']) }}"
                                                                        data-active="{{ $lv3['id'] }}"
                                                                        class="nav-link">
                                                                        {{ $lv3['name'] }}</a>
                                                                </li>
                                                            @endif
                                                        @endforeach
                                                    </ul>
                                                </div>
                                            </li>
                                        @else
                                            <li class="nav-item ">
                                                <a href="{{ URL::to($lv2['route']) }}"
                                                    data-active="{{ $lv2['id'] }}"
                                                    class="nav-link">{{ $lv2['name'] }}
                                                </a>
                                            </li>
                                        @endif
                                    @endforeach
                                </ul>
                            </div>
                        </li>
                    @else
                        <li class="nav-item">
                            <a class="nav-link menu-link" data-active="{{ $lv1['id'] }}"
                                href="{{ URL::to($lv1['route']) }}">
                                <i class="ri-{{ $lv1['icon'] }}"></i> <span>{{ $lv1['name'] }}</span>
                            </a>
                        </li>
                    @endif
                @endforeach


                @isset($userChangers)
                    <li class="nav-item">
                        <div class="nav-link">
                            <div class="col-12">
                                <div class="d-flex align-items-center">
                                    <select class="form-control" id="userChangers">
                                        <option value="">Pilih Users</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </li>
                @endisset
            </ul>
        </div>
        <!-- Sidebar -->
    </div>

    <div class="sidebar-background"></div>
</div>
