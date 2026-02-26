@extends('layout.main')
<script src="https://maps.googleapis.com/maps/api/js?key=AIzaSyBNrDMFbgKfbIxSpFadAk7YjcJ9qTNwYU8" async defer></script>
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card shadow-lg rounded-4 border-0 mb-4">
                <div class="card-header bg-white border-0 pb-0">
                    <div class="row align-items-center g-2">
                        <div class="col-md-8">
                            <h4 class="mb-0 fw-bold" style="color:#1e5631;letter-spacing:1px;">PETA SEBARAN SATKER</h4>
                        </div>
                        <div class="col-md-4 text-end">
                            <button class="btn btn-outline-warning btn-sm" id="resetFilter" title="Reset Filter"><i class="ri-refresh-line"></i> Reset</button>
                        </div>
                    </div>
                    <div class="row mt-3">
                        <select class="form-control select2" id="kdsatker_keu" style="width:100%;">
                            <option value="">Pilih Satker</option>
                            {!! $satkerOptions !!}
                        </select>
                    </div>
                </div>
                <div class="card-body">
                    <div id="gmap" style="width:100%; height: 450px; border-radius:18px; box-shadow:0 4px 24px #1e563122; transition:box-shadow 0.3s;"></div>
                </div>
            </div>
        </div>
    </div>
    <button id="btnBackToNational" class="btn btn-outline-warning btn-sm" style="display:none;position:absolute;top:90px;left:40px;z-index:1001;">Kembali ke Peta Nasional</button>
    <div id="mapsModal" class="modal fade" tabindex="-1" data-bs-focus="false" aria-labelledby="mapsModalLabel" aria-hidden="true">
        <div class="modal-dialog modal-xl">
            <div class="modal-content animate__animated animate__fadeInDown">
                <div class="modal-header">
                    <h5 class="modal-title" id="mapsModalLabel">Detail Data Aset Pemetaan</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
                </div>
                <div class="modal-body" id="modalencuk">
                    <div class="row">
                        <div class="col-lg-12">
                            <div id="gmap" style="width:100%;height:400px;"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
<style>
    #gmap { background: #f9f9f9; }
    .gm-style .gm-style-iw-c { border-radius: 16px !important; box-shadow: 0 4px 24px #1e563122 !important; }
    .select2-container .select2-selection--single { height: 38px; border-radius: 12px; border: 1.5px solid #f9b233; }
    .select2-container--default .select2-selection--single .select2-selection__rendered { color: #1e5631; line-height: 38px; }
    .select2-container--default .select2-selection--single .select2-selection__arrow { height: 38px; }
    .modal-content.animate__animated { animation-duration: 0.5s; }
    @media (max-width: 600px) {
        #gmap { height: 250px !important; }
        .modal-xl { max-width: 98vw; }
    }
</style>
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/animate.css/4.1.1/animate.min.css"/>
@endsection
@section('js')
<script src="https://cdn.jsdelivr.net/npm/select2@4.1.0-rc.0/dist/js/select2.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>
<script>
    $(function() {
        $('#kdsatker_keu').select2({
            placeholder: 'Pilih Satker',
            allowClear: true,
            width: '100%'
        });
        $('#resetFilter').on('click', function() {
            $('#kdsatker_keu').val('').trigger('change');
            // Reset peta ke tampilan nasional dengan animasi zoom out
            let targetZoom = 5;
            let currentZoom = mapnya.getZoom();
            let centerTarget = new google.maps.LatLng(-1.605328, 117.451067);
            let zoomStep = currentZoom > targetZoom ? -1 : 1;
            let zoomAnim = setInterval(function() {
                currentZoom = mapnya.getZoom();
                if ((zoomStep < 0 && currentZoom <= targetZoom) || (zoomStep > 0 && currentZoom >= targetZoom)) {
                    mapnya.setZoom(targetZoom);
                    mapnya.panTo(centerTarget);
                    clearInterval(zoomAnim);
                    renderWilayahMarkers();
                    $('#btnBackToNational').hide();
                } else {
                    mapnya.setZoom(currentZoom + zoomStep);
                    mapnya.panTo(centerTarget);
                }
            }, 180);
        });
        var locations = [
            @foreach ($gps as $kordinat)
                [ '{{ $kordinat->inst_satkerkd }}', {{ $kordinat->lat }}, {{ $kordinat->long }}, "{{ url('/assets/images/icon-kejaksaan.png') }}", "{{ $kordinat->inst_nama|e('js') }}" ],  
            @endforeach
        ];
        var myOptions = {
            zoom: 5,
            center: new google.maps.LatLng(-1.605328, 117.451067),
            disableDefaultUI: true,
            zoomControl: true,
            mapTypeControl: false,
            zoomControlOptions: { style: google.maps.ZoomControlStyle.SMALL },
            draggingCursor: 'move'
        }
        var mapnya = new google.maps.Map(document.getElementById("gmap"), myOptions);
        var gmarkers = [];
        var infowindow = new google.maps.InfoWindow();
        function renderWilayahMarkers() {
            clearMarkers();
            for (let i = 0; i < locations.length; i++) {
                let marker = new google.maps.Marker({
                    position: new google.maps.LatLng(locations[i][1], locations[i][2]),
                    map: mapnya,
                    icon: locations[i][3],
                    title: locations[i][4],
                    animation: google.maps.Animation.DROP
                });
                gmarkers.push(marker);
                google.maps.event.addListener(marker, 'mouseover', (function(marker, i) {
                    return function() {
                        marker.setAnimation(google.maps.Animation.BOUNCE);
                        infowindow.setContent('<b>'+locations[i][4]+'</b>');
                        infowindow.open(mapnya, marker);
                    }
                })(marker, i));
                google.maps.event.addListener(marker, 'mouseout', (function(marker, i) {
                    return function() {
                        marker.setAnimation(null);
                        infowindow.close();
                    }
                })(marker, i));
                google.maps.event.addListener(marker, 'click', (function(marker, i) {
                    return function() {
                        zoomToWilayah(locations[i][0], locations[i][1], locations[i][2], locations[i][4]);
                    }
                })(marker, i));
            }
        }
        renderWilayahMarkers();
        function renderSatkerMarkers(satkerList) {
            clearMarkers();
            for (let i = 0; i < satkerList.length; i++) {
                let marker = new google.maps.Marker({
                    position: new google.maps.LatLng(satkerList[i]['lat'], satkerList[i]['long']),
                    map: mapnya,
                    icon: "{{ url('/assets/images/marker.png') }}",
                    title: satkerList[i]['nama'],
                    animation: google.maps.Animation.DROP
                });
                gmarkers.push(marker);
                let infowindow = new google.maps.InfoWindow();
                google.maps.event.addListener(marker, 'mouseover', (function(marker, i) {
                    return function() {
                        marker.setAnimation(google.maps.Animation.BOUNCE);
                        infowindow.setContent('<b>'+satkerList[i]['nama']+'</b>');
                        infowindow.open(mapnya, marker);
                    }
                })(marker, i));
                google.maps.event.addListener(marker, 'mouseout', (function(marker, i) {
                    return function() {
                        marker.setAnimation(null);
                        infowindow.close();
                    }
                })(marker, i));
                google.maps.event.addListener(marker, 'click', (function(marker, i) {
                    return function() {
                        showSatkerDetailModal(satkerList[i]['kdsatker'], satkerList[i]['nama']);
                    }
                })(marker, i));
            }
        }
        function zoomToWilayah(kdsatker, lat, lng, wilayahNama) {
            $.LoadingOverlay("show");
            mapnya.setZoom(9);
            mapnya.panTo(new google.maps.LatLng(lat, lng));
            $('#btnBackToNational').show();
            $.ajax({
                type: "POST",
                url: `{{ url('/asset/sebaran/getsatkerkoordinat') }}`,
                data: { 'kdsatker': kdsatker },
                success: function(resp) {
                    renderSatkerMarkers(resp.kejari);
                    $.LoadingOverlay("hide", true);
                },
                error: function() {
                    $.LoadingOverlay("hide", true);
                    Swal.fire({ icon: 'error', title: 'Gagal', text: 'Gagal memuat sebaran wilayah!', toast: true, position: 'top-end', timer: 2500, showConfirmButton: false });
                }
            });
        }
        $('#btnBackToNational').on('click', function() {
            let targetZoom = 5;
            let currentZoom = mapnya.getZoom();
            let centerTarget = new google.maps.LatLng(-1.605328, 117.451067);
            let zoomStep = currentZoom > targetZoom ? -1 : 1;
            let zoomAnim = setInterval(function() {
                currentZoom = mapnya.getZoom();
                if ((zoomStep < 0 && currentZoom <= targetZoom) || (zoomStep > 0 && currentZoom >= targetZoom)) {
                    mapnya.setZoom(targetZoom);
                    mapnya.panTo(centerTarget);
                    clearInterval(zoomAnim);
                    renderWilayahMarkers();
                    $('#btnBackToNational').hide();
                } else {
                    mapnya.setZoom(currentZoom + zoomStep);
                    mapnya.panTo(centerTarget);
                }
            }, 180);
        });
        
        // Reset ke peta nasional saat modal ditutup
        $('#mapsModal').on('hidden.bs.modal', function () {
            mapnya.setZoom(5);
            mapnya.panTo(new google.maps.LatLng(-1.605328, 117.451067));
            renderWilayahMarkers();
            $('#btnBackToNational').hide();
        });
        $('#kdsatker_keu').on('change', function(){
            $.LoadingOverlay("show");
            $.ajax({
                type: "POST",
                url: `{{ url('/asset/sebaran/getsatkerkoordinat') }}`,
                data: { 'kdsatker':$(this).val() },
                success: function(resp){
                    mapnya.setZoom(10);
                    mapnya.panTo(new google.maps.LatLng(resp.latitude, resp.longitude));
                    renderSatkerMarkers(resp.kejari);
                    $('#btnBackToNational').show();
                    $.LoadingOverlay("hide", true);
                },
                error: function() {
                    $.LoadingOverlay("hide", true);
                    Swal.fire({ icon: 'error', title: 'Gagal', text: 'Gagal memuat data satker!', toast: true, position: 'top-end', timer: 2500, showConfirmButton: false });
                }
            });
        });
        function setMapOnAll(map) { for (var i = 0; i < gmarkers.length; i++) { gmarkers[i].setMap(map); } }
        function clearMarkers() { setMapOnAll(null); }
        function showSatkerDetailModal(kdsatker, wilayahNama) {
            $.LoadingOverlay("show");
            $('#mapsModalLabel').text('Detail Aset Satuan Kerja di ' + wilayahNama);
            $('#modalencuk').html('<div class="text-center py-5"><div class="spinner-border text-warning" role="status"></div></div>');
            $('#mapsModal').modal('show');
            $.ajax({
                type: "POST",
                url: `{{ url('/asset/sebaran/getsatkerkoordinat') }}`,
                data: { 'kdsatker': kdsatker },
                success: function(resp) {
                    let mapHtml = '<div id="gmapdet" style="width:100%; height: 400px; border-radius:16px;"></div>';
                    $('#modalencuk').html(mapHtml);
                    var myOptions = {
                        zoom: 9,
                        center: new google.maps.LatLng(resp.latitude, resp.longitude),
                        disableDefaultUI: true,
                        zoomControl: true,
                        draggingCursor: 'move'
                    };
                    var map = new google.maps.Map(document.getElementById("gmapdet"), myOptions);
                    for(var i = 0; i < resp.kejari.length; i++) {
                        let marker = new google.maps.Marker({
                            position: new google.maps.LatLng(resp.kejari[i]['lat'], resp.kejari[i]['long']),
                            map: map,
                            icon: "{{ url('/assets/images/marker.png') }}",
                            title: resp.kejari[i]['nama'],
                            animation: google.maps.Animation.DROP
                        });
                        let infowindow = new google.maps.InfoWindow();
                        google.maps.event.addListener(marker, 'mouseover', (function(marker, i) {
                            return function() {
                                marker.setAnimation(google.maps.Animation.BOUNCE);
                                infowindow.setContent('<b>'+resp.kejari[i]['nama']+'</b>');
                                infowindow.open(map, marker);
                            }
                        })(marker, i));
                        google.maps.event.addListener(marker, 'mouseout', (function(marker, i) {
                            return function() {
                                marker.setAnimation(null);
                                infowindow.close();
                            }
                        })(marker, i));
                    }
                    $.LoadingOverlay("hide", true);
                },
                error: function() {
                    $.LoadingOverlay("hide", true);
                    Swal.fire({ icon: 'error', title: 'Gagal', text: 'Gagal memuat detail aset satker!', toast: true, position: 'top-end', timer: 2500, showConfirmButton: false });
                }
            });
        }
    });
</script>
@endsection
