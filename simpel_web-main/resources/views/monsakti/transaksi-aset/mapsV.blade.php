

<div class="row">
    <div class="col-lg-12">
        <div class="card">
            <div class="card-header">
                <div class="d-flex align-items-center">
                    <div class="flex-grow-1">
                        <h5 class="card-title mb-0">Pointing Geolokasi Asset</h5>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <div class="row">
                    <div class="col-lg-6">
                        <div class="row">
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">GPS Longitude</label>
                                    <input type="text" readonly class="form-control" id="gps_longitude" name="gps_longitude" value="{{ $model['gps_longitude'] ?? '' }}">
                                </div>
                            </div>
                            <div class="col-lg-6">
                                <div class="mb-3">
                                    <label for="name" class="form-label">GPS Latitude</label>
                                    <input type="text" readonly class="form-control" id="gps_latitude" name="gps_latitude" value="{{ $model['gps_latitude'] ?? '' }}">
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <!-- <a href="javascript:void(0);" class="btn btn-success">Pointing Koordinat GPS</a> -->
                                <button id="mappoint" class="btn btn-success btn-label waves-effect waves-light">
                                    <i class="ri-add-line label-icon align-middle fs-16 me-2"></i> Pointing Koordinat GPS
                                </button>
                            </div>
                        </div>
                    </div>
                    <div class="col-lg-6">


                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<div id="mapsModal" class="modal fade" tabindex="-1" data-bs-focus="false" aria-labelledby="mapsModalLabel" aria-hidden="true" style="display: none;">
    <div class="modal-dialog modal-lg">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title" id="mapsModalLabel">Pointing Koordinat GPS</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"> </button>
            </div>
            <div class="modal-body" id="modalencuk">

                <div class="row">
                    <div class="col-lg-12">
                        <div id="gmap" style="width:100%;height:500px;"></div>
                    </div>
                </div>

            </div>
        </div><!-- /.modal-content -->
    </div><!-- /.modal-dialog -->
</div><!-- /.modal -->

<script>
    $(function() {

        $('#mappoint').on('click', function(){
            
            $.LoadingOverlay("show");
            $('#modalencuk').html('');
            $('#mapsModal').modal('show');

            $('#mapsModal').on('shown.bs.modal', function (e) {
                e.preventDefault();
                $.LoadingOverlay("show");
                $('#modalencuk').html('');
                $.ajax({
                    type: "POST",
                    url: `{{ url('/mapsdetail') }}`,
                    data: {
                        'gps_longitude':$('#gps_longitude').val(),
                        'gps_latitude':$('#gps_latitude').val(),
                        'jenis_aset' : `{{ $jenis_aset }}`,
                        'id_aset' : `{{ $model['id'] ?? '0' }}`,
                    },
                    success: function(resp){
                        $('#modalencuk').html(resp);
                        $.LoadingOverlay("hide", true);
                    }
                });
            });
            
            $(this).off('shown.bs.modal');
        });

    });

    
</script>
