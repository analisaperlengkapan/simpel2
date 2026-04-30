<style>
 	#gmap {
    width: 100%;
    height: 500px;
 	}

  .modal-lg{
    width: 80%!important;
    padding: 10px!important;
  }
</style>

<div class="row">
    <div class="col-lg-12">
      <div id="gmap"></div>
    </div>
    <div class="col-lg-12">&nbsp;</div>
    <div class="col-lg-6">
      <div class="mb-3">
          <label for="name" class="form-label">Longitude</label>
          <input type="text" readonly class="form-control" id="longitude" name="longitude" value="{{ $gps_longitude ?? '' }}">
      </div>
    </div>
    <div class="col-lg-6">
      <div class="mb-3">
          <label for="name" class="form-label">Latitude</label>
          <input type="text" readonly class="form-control" id="latitude" name="latitude" value="{{ $gps_latitude ?? '' }}">
      </div>
    </div>
    <div class="col-lg-12">
      <button id="simpanmaps" class="btn btn-danger btn-label waves-effect waves-light">
        <i class="ri-add-line label-icon align-middle fs-16 me-2"></i>Simpan Data
      </button>
    </div>
</div>


<script>
  var myOptions = {
      zoom: 10,
      center: new google.maps.LatLng(0.9744875449783096, 120.69811545443805),
      disableDefaultUI: true,
      zoomControl: true,
      draggingCursor: 'move'
  }

  var map = new google.maps.Map(document.getElementById("gmap"), myOptions);

  var marker = "";
  function placeMarker(location) {
    if (marker) {
      marker.setPosition(location);
    } else {
      marker = new google.maps.Marker({
        position: location,
        map: map,
        icon: "{{ url('/assets/images/marker.png') }}",
      });
    }
    
    //var koordinat = location.lng()+"|"+location.lat();
    //$("#koordinat").val(koordinat);

    $('#longitude').val( location.lng() );
    $('#latitude').val( location.lat() );
  }

  google.maps.event.addListener(map, 'click', function(event) {
    placeMarker(event.latLng);
  });

  $('#simpanmaps').on('click', function(){
    $.LoadingOverlay("show");
    $.ajax({
        type: "POST",
        url: `{{ url('/mapssimpan') }}`,
        data: {
            'gps_longitude':$('#longitude').val(),
            'gps_latitude':$('#latitude').val(),
            'jenis_aset' : `{{ $jenis_aset }}`,
            'id' : `{{ $id ?? '0' }}`,
        },
        success: function(resp){
            //$('#modalencuk').html(resp);
            // $('#mapsModal').modal('hide');
            // $('#gps_longitude').val( $('#longitude').val() );
            // $('#gps_latitude').val( $('#latitude').val() );
            // $.LoadingOverlay("hide", true);

            notify({
                type: "success",
                message: "Data Berhasil Disimpan",
            });
        }
    }).done(function( msg ) {
        // dt.ajax.reload();
        // $('#myModal').modal('hide');

        $('#mapsModal').modal('hide');
        $('#gps_longitude').val( $('#longitude').val() );
        $('#gps_latitude').val( $('#latitude').val() );
        $.LoadingOverlay("hide", true);
    });
  });
</script>