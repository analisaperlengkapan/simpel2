<!doctype html>
<html lang="en" data-layout="vertical" data-topbar="light" data-sidebar="dark" data-sidebar-size="lg"
    data-sidebar-image="none" data-preloader="disable">

<head>

    <meta charset="utf-8" />
    <title>{{ config('app.name') }}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta content="Premium Multipurpose Admin & Dashboard Template" name="description" />
    <meta content="Themesbrand" name="author" />
    <!-- App favicon -->
    <link rel="shortcut icon" href="{{ url('/assets/images/favicon.ico') }}">

    <!-- Layout config Js -->
    <script src="{{ url('/assets/js/layout.js') }}"></script>
    <!-- Bootstrap Css -->
    <link href="{{ url('/assets/css/bootstrap.min.css') }}" rel="stylesheet" type="text/css" />
    <!-- Icons Css -->
    <link href="{{ url('/assets/css/icons.min.css') }}" rel="stylesheet" type="text/css" />
    <!-- App Css-->
    <link href="{{ url('/assets/css/app.min.css') }}" rel="stylesheet" type="text/css" />
    <!-- custom Css-->
    <link href="{{ url('/assets/css/custom.min.css') }}" rel="stylesheet" type="text/css" />
    <style>
        body {
            background: #fff7e0 !important;
        }
        .auth-page-wrapper, .auth-page-content {
            background: #fff7e0 !important;
        }
        .card, .error-basic-img {
            border: 2.5px solid #f9b233 !important;
            background: #fff !important;
            border-radius: 18px;
            box-shadow: 0 4px 24px 0 #1e563122;
        }
        h1, h2, h3, h4, h5, h6 {
            color: #1e5631 !important;
        }
        .btn-primary, .btn-success, .btn-info {
            background: #1e5631 !important;
            border-color: #1e5631 !important;
            color: #fff !important;
            border-radius: 12px;
            box-shadow: 0 2px 8px 0 #1e563122;
        }
        .btn-primary:hover, .btn-success:hover, .btn-info:hover {
            background: #f9b233 !important;
            border-color: #f9b233 !important;
            color: #1e5631 !important;
        }
        .footer {
            background: #fff !important;
            border-top: 2.5px solid #f9b233;
            border-radius: 18px 18px 0 0;
        }
    </style>

</head>

<body>
    <div class="auth-page-wrapper pt-5">
        <!-- auth page bg -->
        <div class="auth-one-bg-position auth-one-bg" id="auth-particles">
            <div class="bg-overlay"></div>

            <div class="shape">
                <svg xmlns="http://www.w3.org/2000/svg" version="1.1" xmlns:xlink="http://www.w3.org/1999/xlink"
                    viewBox="0 0 1440 120">
                    <path d="M 0,36 C 144,53.6 432,123.2 720,124 C 1008,124.8 1296,56.8 1440,40L1440 140L0 140z"></path>
                </svg>
            </div>
        </div>

        <!-- auth page content -->
        <div class="auth-page-content">
            <div class="container">
                <div class="row">
                    <div class="col-lg-12">
                        <div class="text-center pt-4">
                            <div class="">
                                <img src="{{ url('assets/images/error.svg') }}" alt=""
                                    class="error-basic-img move-animation">
                            </div>
                            @yield('content')
                        </div>
                    </div>
                </div>
                <!-- end row -->

            </div>
            <!-- end container -->
        </div>
        <!-- end auth page content -->

        <!-- footer -->
        <footer class="footer">
            <div class="container">
                <div class="row">
                    <div class="col-lg-12">
                        <div class="text-center">
                            <p class="mb-0 text-muted">&copy;
                                <script>
                                    document.write(new Date().getFullYear())
                                </script> Velzon. Crafted with <i class="mdi mdi-heart text-danger"></i>
                                by Themesbrand
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
        <!-- end Footer -->

    </div>

    <!-- Theme Settings -->

    <!-- JAVASCRIPT -->
    <script src="{{ url('/assets/libs/bootstrap/js/bootstrap.bundle.min.js') }}"></script>
    <script src="{{ url('/assets/libs/simplebar/simplebar.min.js') }}"></script>
    <script src="{{ url('/assets/libs/node-waves/waves.min.js') }}"></script>
    <script src="{{ url('/assets/libs/feather-icons/feather.min.js') }}"></script>
    <script src="{{ url('/assets/js/pages/plugins/lord-icon-2.1.0.js') }}"></script>
    <script src="{{ url('/assets/js/plugins.js') }}"></script>

    <!-- App js -->
    <script src="{{ url('/assets/js/app.js') }}"></script>
</body>

</html>
