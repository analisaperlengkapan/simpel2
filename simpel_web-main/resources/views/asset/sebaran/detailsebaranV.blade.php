<div class="row">
    <div class="col-12">
        <div class="card">
            <div class="card-header">
                <h5 class="card-title mb-0"> {{ $satker->inst_nama ?? '' }} </h5>
            </div>
            <div class="card-body">
                <!-- <h4>STATISTIK JUMLAH ASSET BMN KEJAKSAAN RI</h4>
                <hr/> -->
                <div class="row">
                    @foreach ($statistik as $rows)
                    <div class="col-xl-4 col-md-4">
                        <div class="card card-animate overflow-hidden">
                            <div class="position-absolute start-0" style="z-index: 0;">
                                <svg version="1.2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 120" width="200" height="120">
                                    <style>
                                        .s0 {
                                            opacity: .05;
                                            fill: var(--vz-success)
                                        }
                                    </style>
                                    <path id="Shape 8" class="s0" d="m189.5-25.8c0 0 20.1 46.2-26.7 71.4 0 0-60 15.4-62.3 65.3-2.2 49.8-50.6 59.3-57.8 61.5-7.2 2.3-60.8 0-60.8 0l-11.9-199.4z" />
                                </svg>
                            </div>
                            <div class="card-body" style="z-index:1 ;">
                            <a href="{{ url($rows->url) }}">
                                <div class="d-flex align-items-center">
                                    <div class="flex-grow-1 overflow-hidden">
                                        <p class="text-uppercase fw-medium text-muted text-truncate mb-3"> {{ $rows->kategori }} </p>
                                        <h4 class="fs-22 fw-semibold ff-secondary mb-0"><span class="counter-value" data-target="{{ $rows->total }}">{{ $rows->total }}</span></h4>
                                    </div>
                                    <div class="flex-shrink-0">
                                        <div id="total_jobs" data-colors='["--vz-success"]' class="apex-charts" dir="ltr"></div>
                                    </div>
                                </div>
                            </a>
                            </div><!-- end card body -->
                        </div><!-- end card -->
                    </div><!--end col-->
                    @endforeach
                </div>

            </div>
        </div>
    </div>

</div>

<div class="row">
    <div class="col-12">
        <div class="card">
            <div class="card-header">
                <h5 class="card-title mb-0"> Peta Sebaran Asset </h5>
            </div>
            <div class="card-body">
                <div id="gmapdet" style="width:100%; height: 550px;"></div>
            </div>
        </div>
    </div>
</div>

<script>
    var myOptions = {
        zoom: 10,
        center: new google.maps.LatLng( {{ $latitude }} , {{ $longitude }} ),
        disableDefaultUI: true,
        zoomControl: true,
        draggingCursor: 'move'
    }

    var map = new google.maps.Map(document.getElementById("gmapdet"), myOptions);
</script>