@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengaturan Tema Aplikasi</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <div class="offcanvas-body p-0">
                        <div data-simplebar class="h-100">
                            <div class="p-4">
                                
                                <div class="row">
                                    <div class="col-12">
                                        <h6 class="mt-4 mb-0 fw-semibold text-uppercase">Color Scheme</h6>
                                        <p class="text-muted">Choose Light or Dark Scheme.</p>

                                         <div class="colorscheme-cardradio">
                                             <div class="row">
                                                 <div class="col-6">
                                                     <div class="form-check card-radio">
                                                         <input class="form-check-input" type="radio" name="data-layout-mode" id="layout-mode-light" value="light">
                                                         <label class="form-check-label p-0 avatar-md w-100" for="layout-mode-light">
                                                             <span class="d-flex gap-1 h-100">
                                                                 <span class="flex-shrink-0">
                                                                     <span class="bg-light d-flex h-100 flex-column gap-1 p-1">
                                                                         <span class="d-block p-1 px-2 bg-soft-primary rounded mb-2"></span>
                                                                         <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                         <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                         <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                     </span>
                                                                 </span>
                                                                 <span class="flex-grow-1">
                                                                     <span class="d-flex h-100 flex-column">
                                                                         <span class="bg-light d-block p-1"></span>
                                                                         <span class="bg-light d-block p-1 mt-auto"></span>
                                                                     </span>
                                                                 </span>
                                                             </span>
                                                         </label>
                                                     </div>
                                                     <h5 class="fs-13 text-center mt-2">Light</h5>
                                                 </div>

                                                 <div class="col-6">
                                                     <div class="form-check card-radio dark">
                                                         <input class="form-check-input" type="radio" name="data-layout-mode" id="layout-mode-dark" value="dark">
                                                         <label class="form-check-label p-0 avatar-md w-100 bg-dark" for="layout-mode-dark">
                                                             <span class="d-flex gap-1 h-100">
                                                                 <span class="flex-shrink-0">
                                                                     <span class="bg-soft-light d-flex h-100 flex-column gap-1 p-1">
                                                                         <span class="d-block p-1 px-2 bg-soft-light rounded mb-2"></span>
                                                                         <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                         <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                         <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                     </span>
                                                                 </span>
                                                                 <span class="flex-grow-1">
                                                                     <span class="d-flex h-100 flex-column">
                                                                         <span class="bg-soft-light d-block p-1"></span>
                                                                         <span class="bg-soft-light d-block p-1 mt-auto"></span>
                                                                     </span>
                                                                 </span>
                                                             </span>
                                                         </label>
                                                     </div>
                                                     <h5 class="fs-13 text-center mt-2">Dark</h5>
                                                 </div>
                                             </div>
                                         </div>

                                         <hr/>

                                         <h6 class="mt-4 mb-0 fw-semibold text-uppercase">Topbar Color</h6>
                                         <p class="text-muted">Choose Light or Dark Topbar Color.</p>

                                            <div class="row">
                                                <div class="col-6">
                                                    <div class="form-check card-radio">
                                                        <input class="form-check-input" type="radio" name="data-topbar" id="topbar-color-light" value="light">
                                                        <label class="form-check-label p-0 avatar-md w-100" for="topbar-color-light">
                                                            <span class="d-flex gap-1 h-100">
                                                                <span class="flex-shrink-0">
                                                                    <span class="bg-light d-flex h-100 flex-column gap-1 p-1">
                                                                        <span class="d-block p-1 px-2 bg-soft-primary rounded mb-2"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                    </span>
                                                                </span>
                                                                <span class="flex-grow-1">
                                                                    <span class="d-flex h-100 flex-column">
                                                                        <span class="bg-light d-block p-1"></span>
                                                                        <span class="bg-light d-block p-1 mt-auto"></span>
                                                                    </span>
                                                                </span>
                                                            </span>
                                                        </label>
                                                    </div>
                                                    <h5 class="fs-13 text-center mt-2">Light</h5>
                                                </div>
                                                <div class="col-6">
                                                    <div class="form-check card-radio">
                                                        <input class="form-check-input" type="radio" name="data-topbar" id="topbar-color-dark" value="dark">
                                                        <label class="form-check-label p-0 avatar-md w-100" for="topbar-color-dark">
                                                            <span class="d-flex gap-1 h-100">
                                                                <span class="flex-shrink-0">
                                                                    <span class="bg-light d-flex h-100 flex-column gap-1 p-1">
                                                                        <span class="d-block p-1 px-2 bg-soft-primary rounded mb-2"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                    </span>
                                                                </span>
                                                                <span class="flex-grow-1">
                                                                    <span class="d-flex h-100 flex-column">
                                                                        <span class="bg-primary d-block p-1"></span>
                                                                        <span class="bg-light d-block p-1 mt-auto"></span>
                                                                    </span>
                                                                </span>
                                                            </span>
                                                        </label>
                                                    </div>
                                                    <h5 class="fs-13 text-center mt-2">Dark</h5>
                                                </div>
                                            </div>

                                            <hr/>

                                            <h6 class="mt-4 mb-0 fw-semibold text-uppercase">Sidebar Color</h6>
                                            <p class="text-muted">Choose a color of Sidebar.</p>
                                            <div class="row">
                                                <div class="col-6">
                                                    <div class="form-check sidebar-setting card-radio" data-bs-toggle="collapse" data-bs-target="#collapseBgGradient.show">
                                                        <input class="form-check-input" type="radio" name="data-sidebar" id="sidebar-color-light" value="light">
                                                        <label class="form-check-label p-0 avatar-md w-100" for="sidebar-color-light">
                                                            <span class="d-flex gap-1 h-100">
                                                                <span class="flex-shrink-0">
                                                                    <span class="bg-white border-end d-flex h-100 flex-column gap-1 p-1">
                                                                        <span class="d-block p-1 px-2 bg-soft-primary rounded mb-2"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-primary"></span>
                                                                    </span>
                                                                </span>
                                                                <span class="flex-grow-1">
                                                                    <span class="d-flex h-100 flex-column">
                                                                        <span class="bg-light d-block p-1"></span>
                                                                        <span class="bg-light d-block p-1 mt-auto"></span>
                                                                    </span>
                                                                </span>
                                                            </span>
                                                        </label>
                                                    </div>
                                                    <h5 class="fs-13 text-center mt-2">Light</h5>
                                                </div>
                                                <div class="col-6">
                                                    <div class="form-check sidebar-setting card-radio" data-bs-toggle="collapse" data-bs-target="#collapseBgGradient.show">
                                                        <input class="form-check-input" type="radio" name="data-sidebar" id="sidebar-color-dark" value="dark">
                                                        <label class="form-check-label p-0 avatar-md w-100" for="sidebar-color-dark">
                                                            <span class="d-flex gap-1 h-100">
                                                                <span class="flex-shrink-0">
                                                                    <span class="bg-primary d-flex h-100 flex-column gap-1 p-1">
                                                                        <span class="d-block p-1 px-2 bg-soft-light rounded mb-2"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                        <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                    </span>
                                                                </span>
                                                                <span class="flex-grow-1">
                                                                    <span class="d-flex h-100 flex-column">
                                                                        <span class="bg-light d-block p-1"></span>
                                                                        <span class="bg-light d-block p-1 mt-auto"></span>
                                                                    </span>
                                                                </span>
                                                            </span>
                                                        </label>
                                                    </div>
                                                    <h5 class="fs-13 text-center mt-2">Dark</h5>
                                                </div>
                                                <!-- <div class="col-4">
                                                    <button class="btn btn-link avatar-md w-100 p-0 overflow-hidden border collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#collapseBgGradient" aria-expanded="false" aria-controls="collapseBgGradient">
                                                        <span class="d-flex gap-1 h-100">
                                                            <span class="flex-shrink-0">
                                                                <span class="bg-vertical-gradient d-flex h-100 flex-column gap-1 p-1">
                                                                    <span class="d-block p-1 px-2 bg-soft-light rounded mb-2"></span>
                                                                    <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                    <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                    <span class="d-block p-1 px-2 pb-0 bg-soft-light"></span>
                                                                </span>
                                                            </span>
                                                            <span class="flex-grow-1">
                                                                <span class="d-flex h-100 flex-column">
                                                                    <span class="bg-light d-block p-1"></span>
                                                                    <span class="bg-light d-block p-1 mt-auto"></span>
                                                                </span>
                                                            </span>
                                                        </span>
                                                    </button>
                                                    <h5 class="fs-13 text-center mt-2">Gradient</h5>
                                                </div> -->
                                            </div>

                                    </div>
                                </div>

                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>

    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">Pengaturan Logo Aplikasi</h5>
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <form>
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Upload Logo Aplikasi Besar (Light) </label>
                                    <input type="file" id="logo" name="logo" class="form-control"/>
                                    @if ($logotema['logo_aplikasi'] != "")
                                        <a href="{{ url($logotema['logo_aplikasi'] ?? '')  }}" target="_blank"><i class="ri-download-cloud-line"></i> Download</a>
                                    @endif
                                </div> 
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Upload Logo Aplikasi Besar (Dark)</label>
                                    <input type="file" id="logo_dark" name="logo_dark" class="form-control"/>
                                    @if(isset($logotema['logo_aplikasi_dark']))
                                        <a href="{{ url($logotema['logo_aplikasi_dark'] ?? '')  }}" target="_blank"><i class="ri-download-cloud-line"></i> Download</a>
                                    @endif
                                </div> 
                            </div>
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="satkers" class="form-label">Upload Logo Aplikasi Kecil</label>
                                    <input type="file" id="logo_kecil" name="logo_kecil" class="form-control"/>
                                    @if(isset($logotema['logo_kecil']))
                                        <a href="{{ url($logotema['logo_kecil'] ?? '')  }}" target="_blank"><i class="ri-download-cloud-line"></i> Download</a>
                                    @endif
                                </div> 
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Nama Aplikasi</label>
                                    <input type="text" class="form-control" id="nama_aplikasi" name="nama_aplikasi" value="{{ $logotema['nama_aplikasi'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="nip" class="form-label">Copyright</label>
                                    <input type="text" class="form-control" id="copyright" name="copyright" value="{{ $logotema['copyright'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class=" col-lg-12">
                                <hr/>
                                <button class="btn btn-primary" type="button" id="simpan">Simpan</button>
                            </div>
                        </div>

                    </form>
                </div>
            </div>
        </div>
    </div>
    
<style>
   
</style>
@endsection

@section('js')
<script>
    $(function() {

        $('#simpan').on('click', function() {
            var data = new FormData();
            var files = $('#logo')[0].files;
            var files_dark = $('#logo_dark')[0].files;
            var files_kecil = $('#logo_kecil')[0].files;
            data.append('logo', files[0]);
            data.append('logo_dark', files_dark[0]);
            data.append('logo_kecil', files_kecil[0]);
            data.append('nama_aplikasi', $('#nama_aplikasi').val() );
            data.append('copyright', $('#copyright').val() );

            $.ajax({
                method: "POST",
                url: `{{ $controller }}`,
                data: data,
                processData: false,
                contentType: false,
                success: function(){
                    notify({
                        type: "success",
                        message: "Data Berhasil Disimpan",
                    });

                    location.reload();
                },
                error: showError,
            }).done(function( msg ) {
            });
        });

    });
</script>
@endsection
