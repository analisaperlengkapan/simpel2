<!doctype html>
<html lang="en" data-layout="vertical" data-topbar="light" data-sidebar="dark" data-sidebar-size="lg"
    data-sidebar-image="none" data-preloader="disable">

<head>

    <meta charset="utf-8" />
    <title>{{ getenv('APP_NAME') }}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta content="Premium Multipurpose Admin & Dashboard Template" name="description" />
    <meta content="Themesbrand" name="author" />
    <meta name="csrf-token" content="{{ csrf_token() }}">

    <!-- App favicon -->
    <link rel="shortcut icon" href="{{ url('/assets/images/logo-sm.png') }}">
    <script>
        const baseUrl = `{{ url('') }}`;
    </script>
    <!-- Layout config Js -->
    <script src={{ url('/assets/js/layout.js') }}></script>
    <!-- Bootstrap Css -->
    <link href={{ url('/assets/css/bootstrap.min.css') }} rel="stylesheet" type="text/css" />
    <!-- Icons Css -->
    <link href={{ url('/assets/css/icons.min.css') }} rel="stylesheet" type="text/css" />
    <!-- App Css-->
    <link href={{ url('/assets/css/app.css') }} rel="stylesheet" type="text/css" />
    <!-- custom Css-->
    <link href={{ url('/assets/css/custom.min.css') }} rel="stylesheet" type="text/css" />
    <!--datatable css-->
    <link rel="stylesheet" href="https://cdn.datatables.net/1.11.5/css/dataTables.bootstrap5.min.css" />
    <!--datatable responsive css-->
    <link rel="stylesheet" href="https://cdn.datatables.net/responsive/2.2.9/css/responsive.bootstrap.min.css" />
    <link rel="stylesheet" href="https://cdn.datatables.net/buttons/2.2.2/css/buttons.dataTables.min.css">
    <link rel="stylesheet" href="https://cdn.datatables.net/select/1.7.0/css/select.dataTables.min.css">
    <link href="https://cdn.jsdelivr.net/npm/select2@4.1.0-rc.0/dist/css/select2.min.css" rel="stylesheet" />

    <script src={{ url('/assets/libs/jquery/jquery-3.7.0.min.js') }}></script>

    @if (config('services.google_maps.key'))
        <script src="https://maps.googleapis.com/maps/api/js?key={{ config('services.google_maps.key') }}" async defer>
        </script>
    @endif
    <style>
        body {
            background: #fff7e0 !important;
            transition: background 0.3s;
        }

        #page-topbar,
        .footer {
            background: #fff !important;
            border-bottom: 2.5px solid #f9b233;
            border-radius: 0 0 18px 18px;
            box-shadow: 0 2px 8px 0 #f9b23322;
        }

        .footer {
            border-top: 2.5px solid #f9b233;
            border-radius: 18px 18px 0 0;
        }

        .navbar-menu {
            background: #fff !important;
            border-right: 2.5px solid #f9b233;
            border-radius: 0 18px 18px 0;
            box-shadow: 2px 0 8px 0 #f9b23322;
            transition: width 0.3s;
        }

        .btn-primary,
        .btn-success,
        .btn-info {
            background: #1e5631 !important;
            border-color: #1e5631 !important;
            color: #fff !important;
            border-radius: 12px;
            box-shadow: 0 2px 8px 0 #1e563122;
            transition: background 0.2s;
        }

        .btn-primary:hover,
        .btn-success:hover,
        .btn-info:hover {
            background: #f9b233 !important;
            border-color: #f9b233 !important;
            color: #1e5631 !important;
        }

        .card,
        .modal-content {
            border-radius: 18px;
            box-shadow: 0 4px 24px 0 #1e563122;
        }

        .main-content {
            background: #fff7e0 !important;
            min-height: 100vh;
        }

        .fab {
            position: fixed;
            right: 32px;
            bottom: 32px;
            z-index: 999;
            background: #f9b233;
            color: #1e5631;
            border-radius: 50%;
            width: 56px;
            height: 56px;
            display: flex;
            align-items: center;
            justify-content: center;
            box-shadow: 0 4px 16px 0 #f9b23355;
            font-size: 2rem;
            cursor: pointer;
            transition: background 0.2s;
        }

        .fab:hover {
            background: #1e5631;
            color: #fff;
        }

        .toast-container {
            position: fixed;
            top: 24px;
            right: 24px;
            z-index: 2000;
        }

        .dark-mode body {
            background: #1e5631 !important;
        }

        .dark-mode #page-topbar,
        .dark-mode .footer,
        .dark-mode .navbar-menu,
        .dark-mode .main-content {
            background: #222 !important;
            color: #fff !important;
            border-color: #f9b233;
        }

        .dark-mode .btn-primary,
        .dark-mode .btn-success,
        .dark-mode .btn-info {
            background: #f9b233 !important;
            border-color: #f9b233 !important;
            color: #1e5631 !important;
        }

        .dark-mode .btn-primary:hover,
        .dark-mode .btn-success:hover,
        .dark-mode .btn-info:hover {
            background: #1e5631 !important;
            border-color: #1e5631 !important;
            color: #fff !important;
        }

        .dark-mode .card,
        .dark-mode .modal-content {
            background: #222 !important;
            color: #fff !important;
        }
    </style>
</head>

<body>

    <!-- Begin page -->
    <div id="layout-wrapper">

        <header id="page-topbar">
            <div class="layout-width">
                <div class="navbar-header">
                    <div class="d-flex">
                        <!-- LOGO -->
                        <div class="navbar-brand-box horizontal-logo">
                            <a href="{{ url('') }}" class="logo logo-dark">
                                <span class="logo-sm">
                                    <img src="{{ url('/assets/images/logo_kejaksaan.png') }}" alt=""
                                        height="32">
                                </span>
                                <span class="logo-lg">
                                    <img src="{{ url('/assets/images/logo_kejaksaan.png') }}" alt=""
                                        height="32">
                                </span>
                            </a>
                        </div>
                        <button type="button"
                            class="btn btn-sm px-3 fs-16 header-item vertical-menu-btn topnav-hamburger"
                            id="topnav-hamburger-icon">
                            <span class="hamburger-icon">
                                <span></span>
                                <span></span>
                                <span></span>
                            </span>
                        </button>
                    </div>
                    <div class="d-flex align-items-center">
                        <!-- Dark mode toggle -->
                        <button class="btn btn-outline-warning me-2" id="darkModeToggle" title="Toggle Dark Mode">
                            <i class="ri-moon-line"></i>
                        </button>
                        <div class="dropdown topbar-head-dropdown ms-1 header-item" id="notificationDropdown">
                            <button type="button" class="btn btn-icon btn-topbar btn-ghost-secondary rounded-circle"
                                id="page-header-notifications-dropdown" data-bs-toggle="dropdown"
                                data-bs-auto-close="outside" aria-haspopup="true" aria-expanded="false">
                                <i class='bx bx-bell fs-22'></i>
                                @if ($unreadNotif > 0)
                                    <span
                                        class="position-absolute topbar-badge fs-10 translate-middle badge rounded-pill bg-danger">
                                        {{ $unreadNotif }}
                                        <span class="visually-hidden">unread messages</span>
                                    </span>
                                @endif
                            </button>
                            <div class="dropdown-menu dropdown-menu-lg dropdown-menu-end p-0"
                                aria-labelledby="page-header-notifications-dropdown">

                                <div class="dropdown-head bg-primary bg-pattern rounded-top">
                                    <div class="p-3">
                                        <div class="row align-items-center">
                                            <div class="col">
                                                <h6 class="m-0 fs-16 fw-semibold text-white"> Notifikasi </h6>
                                            </div>
                                            <div class="col-auto dropdown-tabs">
                                                <span class="badge badge-soft-light fs-13">
                                                    {{ $unreadNotif }} New</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="position-relative">
                                    <div data-simplebar style="max-height: 450px;overflow-y:auto" class="pe-2"
                                        id="div-list-notif">
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="dropdown ms-sm-3 header-item topbar-user">
                            <button type="button" class="btn" id="page-header-user-dropdown"
                                data-bs-toggle="dropdown" aria-haspopup="true" aria-expanded="false">
                                <span class="d-flex align-items-center">
                                    <img class="rounded-circle header-profile-user"
                                        src="{{ session('userData.foto') ?? asset('assets/images/no-pic.jpg') }}"
                                        alt="Header Avatar">
                                    <span class="text-start ms-xl-2">
                                        <span
                                            class="d-none d-xl-inline-block ms-1 fw-medium user-name-text">{{ session('userData.name') }}</span>
                                        <div>
                                            <span
                                                class="d-none d-xl-block ms-1 fs-12 text-muted user-name-sub-text">{{ session('userData.current_role.name') }}</span>
                                            <span
                                                class="d-none d-xl-block ms-1 fs-12 text-muted user-name-sub-text">{{ session('userData.current_role.inst_nama', '') }}</span>
                                        </div>
                                    </span>
                                </span>
                            </button>
                            <div class="dropdown-menu dropdown-menu-end">
                                @if (count(session('userData.roles', [])) > 1)
                                    <h6 class="dropdown-header">Ganti Role</h6>
                                    @foreach (session('userData.roles') as $role)
                                        @if ($role->id != session('userData.current_role.id', ''))
                                            <a class="dropdown-item changeRole" href="#"
                                                data-role="{{ $role->id }}">
                                                {{ $role->name }}
                                                {{ $role->inst_nama }}
                                            </a>
                                        @endif
                                    @endforeach
                                @endif
                                <div class="dropdown-divider"></div>
                                <a class="dropdown-item" href="{{ url('/pengguna/profil') }}"><i
                                        class="mdi mdi-account-circle text-muted fs-16 align-middle me-1"></i> <span
                                        class="align-middle">Profile</span></a>
                                <a class="dropdown-item" id="logout-btn" href='#'
                                    data-review="{{ session('userData.has_review') }}"><i
                                        class="mdi mdi-logout text-muted fs-16 align-middle me-1"></i> <span
                                        class="align-middle" data-key="t-logout">Logout</span></a>
                            </div>
                        </div>

                    </div>
                </div>
            </div>
        </header>
        <div id="removeNotificationModal"></div>

        <!-- /.modal -->
        <!-- ========== App Menu ========== -->
        @include('layout.sidebar')
        <!-- Left Sidebar End -->
        <!-- Vertical Overlay-->
        <div class="vertical-overlay"></div>

        <!-- ============================================================== -->
        <!-- Start right Content here -->
        <!-- ============================================================== -->
        <div class="main-content">

            <div class="page-content">
                <div class="container-fluid">
                    <!-- start page title -->
                    @yield('content')
                    <!-- end page title -->

                </div>
                <!-- container-fluid -->
            </div>
            <!-- End Page-content -->

            <footer class="footer">
                <div class="container-fluid">

                    <div class="row">
                        <div class="col-sm-6">
                            <!-- <script>
                                document.write(new Date().getFullYear())
                            </script> © KEJAKSAAN REPUBLIK INDONESIA. -->
                            {{ $copyright ?? '2023 © KEJAKSAAN REPUBLIK INDONESIA.' }}
                        </div>
                        <div class="col-sm-6">
                            <div class="text-sm-end d-none d-sm-block">
                                <!-- SISTEM INFORMASI MANAJEMEN PERLENGKAPAN KEJAKSAAN RI -->
                                {{ $nama_aplikasi ?? 'SISTEM INFORMASI MANAJEMEN PERLENGKAPAN KEJAKSAAN RI' }}
                            </div>
                        </div>
                    </div>
                </div>
            </footer>
        </div>
        <!-- end main content-->

    </div>
    <!-- END layout-wrapper -->

    <!--start back-to-top-->
    <button onclick="topFunction()" class="btn btn-danger btn-icon" id="back-to-top">
        <i class="ri-arrow-up-line"></i>
    </button>
    <!--end back-to-top-->

    <!--preloader-->
    <div id="preloader">
        <div id="status">
            <div class="spinner-border text-primary avatar-sm" role="status">
                <span class="visually-hidden">Loading...</span>
            </div>
        </div>
    </div>

    <!-- Theme Settings -->
    <div id="reviewModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="reviewModal" aria-hidden="true"
        style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">Peniliaian Aplikasi</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="reviewFormModal">
                        <div class="row">
                            <label for="rating" class="form-label">Rating</label>
                            <input type="hidden" name="rating" id="rating">
                            <div dir="ltr" id="rating">
                                <div id="rater-onhover" class="align-middle"></div>
                                <span class="ratingnum badge bg-info align-middle ms-2"></span>
                            </div>
                        </div>
                        <div class="row mt-3">
                            <label for="barang_kode" class="form-label">Review</label>
                            <textarea name="review" id="" cols="30" rows="5" class="form-control"
                                placeholder="Pendapat anda tentang aplikasi ini"></textarea>
                        </div>
                    </form>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" id="skipReview"
                        data-bs-dismiss="modal">Lewati</button>
                    <button type="button" id="sendReview" class="btn btn-primary ">Kirim</button>
                </div>
            </div>
        </div>
    </div>
    @if (session::has('notifMessage'))
        @include('components.popup', ['modalId' => 'popupnotif', 'data' => session('notifMessage')])
    @endif
    <!-- JAVASCRIPT -->
    <script src="https://cdn.jsdelivr.net/npm/select2@4.1.0-rc.0/dist/js/select2.min.js"></script>
    <script src={{ url('/assets/libs/bootstrap/js/bootstrap.bundle.min.js') }}></script>
    <script src={{ url('/assets/libs/simplebar/simplebar.min.js') }}></script>
    <script src={{ url('/assets/libs/node-waves/waves.min.js') }}></script>
    <script src={{ url('/assets/libs/feather-icons/feather.min.js') }}></script>
    <script src={{ url('/assets/libs/apexcharts/apexcharts.min.js') }}></script>
    <script src={{ url('/assets/js/lord-icon-2.1.0.js') }}></script>
    <script src={{ url('/assets/js/plugins.js') }}></script>

    <!-- App js -->
    <script src={{ url('/assets/js/autoNumeric.js') }}></script>
    <script src={{ url('/assets/js/numeral.js') }}></script>
    <script src={{ url('/assets/libs/jquery/jquery-form.min.js') }}></script>
    <script src={{ url('/assets/libs/jquery/jquery-overlay.min.js') }}></script>
    <script src={{ url('/assets/js/custom.js') }}></script>
    <script src="https://unpkg.com/sweetalert/dist/sweetalert.min.js"></script>

    <!--datatable js-->
    <script src="https://cdn.datatables.net/1.11.5/js/jquery.dataTables.min.js"></script>
    <script src="https://cdn.datatables.net/1.11.5/js/dataTables.bootstrap5.min.js"></script>
    <script src="https://cdn.datatables.net/responsive/2.2.9/js/dataTables.responsive.min.js"></script>
    <script src="https://cdn.datatables.net/buttons/2.2.2/js/dataTables.buttons.min.js"></script>
    <script src="https://cdn.datatables.net/buttons/2.2.2/js/buttons.print.min.js"></script>
    <script src="https://cdn.datatables.net/buttons/2.2.2/js/buttons.html5.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdfmake/0.1.53/pdfmake.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdfmake/0.1.53/vfs_fonts.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/jszip/3.1.3/jszip.min.js"></script>
    <script src="https://cdn.datatables.net/select/1.7.0/js/dataTables.select.min.js"></script>
    <script src={{ url('/assets/libs/rater-js/index.js') }}></script>

    @yield('js')
    <script src={{ url('/assets/js/app.js') }}></script>

    <!-- Toast Container -->
    <div class="toast-container" id="toastContainer"></div>
    <script>
        // Dark mode toggle
        document.getElementById('darkModeToggle').onclick = function() {
            document.body.classList.toggle('dark-mode');
        };
        // Tooltip init
        var tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
        tooltipTriggerList.map(function(tooltipTriggerEl) {
            return new bootstrap.Tooltip(tooltipTriggerEl);
        });
        // Toast function
        window.showToast = function(msg, type = 'success') {
            const toast = document.createElement('div');
            toast.className = `toast align-items-center text-bg-${type} border-0 show mb-2`;
            toast.role = 'alert';
            toast.innerHTML =
                `<div class='d-flex'><div class='toast-body'>${msg}</div><button type='button' class='btn-close btn-close-white me-2 m-auto' data-bs-dismiss='toast'></button></div>`;
            document.getElementById('toastContainer').appendChild(toast);
            setTimeout(() => toast.remove(), 4000);
        };
    </script>
    // Memunculkan chat AI
    @include('chat-widget')
</body>
<script>
    const logoutUrl = `{{ url('/auth/logout') }}`;

    $(function() {

        initActiveMenu();
        $('.changeRole').on('click', function() {
            const roleId = $(this).data('role');
            $.get(`/auth/changeRole/${roleId}`)
                .done(res => window.location.href = '/')
                .fail(e => notify({
                    type: 'danger',
                    message: e.responseJSON.message,
                }))
        })
        $('#logout-btn').on('click', function() {
            const hasReview = $(this).data('review');
            if (hasReview == 1) {
                return window.location.href = logoutUrl;
            }
            $('#reviewModal').modal('show');

        })
        raterJs({
            starSize: 22,
            rating: 5,
            element: document.querySelector("#rater-onhover"),
            rateCallback: function rateCallback(rating, done) {
                this.setRating(rating);
                $('#rating').val(rating);
                done();
            },
            onHover: function(currentIndex, currentRating) {
                document.querySelector('.ratingnum').textContent = currentIndex;
            },
            onLeave: function(currentIndex, currentRating) {
                document.querySelector('.ratingnum').textContent = currentRating;
            }
        });
        $('#sendReview').on('click', function() {
            const data = $('#reviewFormModal').serializeFormJSON()
            if (data.rating == '') {
                notify({
                    message: 'Rating Harus diisi!!',
                    'type': 'danger'
                })
                return false;
            }
            const postUrl =
                $.post('/pengguna/review', data, function() {
                    return window.location.href = logoutUrl;
                })
        });

        $('#skipReview').on('click', function() {
            return window.location.href = logoutUrl;
        });
        $('#notificationDropdown')
            .on('show.bs.dropdown', function() {
                $.get('/getNotif').done(data => {
                    let html = data.map(rowNotif);
                    if (html.length < 1) {
                        $('#div-list-notif').html(
                            `<div class="text-reset notification-item d-block dropdown-item position-relative ">
                            <div class="d-flex">
                                <div class="flex-1">
                                        <h5 class="mt-0 mb-0 lh-base font-bold text-center">Belum ada Notifikasi</h5>
                                </div>
                            </div>
                        </div>`
                        );
                        return;
                    }
                    if (data.length > 10) {
                        html +=
                            `<div class="my-3 text-center view-all">
                                <button type="button" class="btn btn-soft-success waves-effect waves-light">Lihat Semua Notifikasi
                                    <i class="ri-arrow-right-line align-middle"></i>
                                </button>
                        </div>`;
                    }
                    $('#div-list-notif').html(html);
                    return;
                });
            }).on('hidden.bs.dropdown', function() {
                $('#div-list-notif').html('');
            })

        $("#userChangers").on('select2:select', function() {
            const username = $(this).val();
            const url = `{{ url('auth') }}/changeUser`;
            $.post(url, {
                username
            }).done(res => location.reload())
        })

        $('#userChangers').select2({
            ajax: {
                url: `{{ url('/getUserChangers') }}`, // Replace with your API endpoint
                dataType: 'json',
                delay: 500,
                data: function(params) {
                    return {
                        q: params.term // Search query term
                    };
                },
                processResults: function(data) {
                    return {
                        results: data.data.map(function(item) {
                            return {
                                id: item.username,
                                text: item.nama_dan_satker
                            };
                        })
                    };
                },
                minimumInputLength: 3
            }
        });
    })

    function rowNotif(notif) {
        let url = `{{ url('${notif.url}') }}`
        const isPusat = notif.ms_satker_id == '00' ? 1 : 0;
        let separator = url.split('?').length > 1 ? '&' : '?';
        const qParam = {
            changeRole: notif.target_role_id,
            notifId: notif.id
        };
        url += `${separator}${$.param(qParam)}`;
        return `<div class="text-reset notification-item d-block dropdown-item position-relative ${notif.is_read == 0 ? 'bg-soft-info' : ''}">
            <div class="d-flex">
                <div class="flex-1">
                    <a href="${url}" class="stretched-link" data-role="${notif.target_role_id}">
                        <h5 class="mt-0 mb-0 lh-base font-bold">${notif.dari_nama}</h5>
                        <h6 class="mt-0 mb-0 lh-base">${notif.dari_role} ${notif.dari_satker || ''}</h6>
                        <p class="mt-0 mb-2 lh-base">${notif.judul}</p>
                    </a>
                    <p class="mb-0 fs-11 fw-medium text-uppercase text-muted">
                        <span><i class="mdi mdi-clock-outline"></i>${dateFormatIndo(notif.created_at)}</span>
                    </p>
                </div>
            </div>
        </div>`
    }

    function initActiveMenu() {

        const activeRoute = `{{ session('activeRoute') }}`;
        if (activeRoute) {
            // navbar-nav
            var a = document.getElementById("navbar-nav").querySelector('[data-active="' + activeRoute + '"]');
            if (a) {
                a.classList.add("active");
                var parentCollapseDiv = a.closest(".collapse.menu-dropdown");
                if (parentCollapseDiv) {
                    parentCollapseDiv.classList.add("show");
                    parentCollapseDiv.parentElement.children[0].classList.add("active");
                    parentCollapseDiv.parentElement.children[0].setAttribute("aria-expanded", "true");
                    if (parentCollapseDiv.parentElement.closest(".collapse.menu-dropdown")) {
                        parentCollapseDiv.parentElement.closest(".collapse").classList.add("show");
                        if (parentCollapseDiv.parentElement.closest(".collapse").previousElementSibling)
                            parentCollapseDiv.parentElement.closest(".collapse").previousElementSibling
                            .classList.add(
                                "active");

                        if (parentCollapseDiv.parentElement.parentElement.parentElement.parentElement.closest(
                                ".collapse.menu-dropdown")) {
                            parentCollapseDiv.parentElement.parentElement.parentElement.parentElement.closest(
                                ".collapse").classList.add("show");
                            if (parentCollapseDiv.parentElement.parentElement.parentElement.parentElement
                                .closest(
                                    ".collapse").previousElementSibling) {

                                parentCollapseDiv.parentElement.parentElement.parentElement.parentElement
                                    .closest(
                                        ".collapse").previousElementSibling.classList.add("active");
                                if ((document.documentElement.getAttribute("data-layout") == "horizontal") &&
                                    parentCollapseDiv.parentElement.parentElement.parentElement.parentElement
                                    .parentElement.parentElement.parentElement.closest(".collapse")) {
                                    parentCollapseDiv.parentElement.parentElement.parentElement.parentElement
                                        .parentElement.parentElement.parentElement.closest(".collapse")
                                        .previousElementSibling.classList.add("active")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
</script>

</html>
