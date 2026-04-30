<div class="row" style="margin-bottom: 25px;">
    <div class="mb-12">
        <div class="accordion accordion-flush" id="accordionFlushExample">
            <div class="accordion-item">
                <h2 class="accordion-header" id="flush-headingOne">
                    <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-collapseOne" aria-expanded="false" aria-controls="flush-collapseOne">
                        Tampilan Kolom
                    </button>
                </h2>
                <div id="flush-collapseOne" class="accordion-collapse collapse" aria-labelledby="flush-headingOne" data-bs-parent="#accordionFlushExample">
                    <div class="accordion-body">
                        <select class="form-control" id="setting-column" data-choices data-choices-removeItem data-choices-sorting-false multiple>
                            @foreach ($columns as $index => $column)
                            @if (in_array($index, $selected))
                                <option data-column="{{ $index }}" value="{{ $index }}" selected>{{ $column }}</option>
                            @else
                                <option data-column="{{ $index }}" value="{{ $index }}">{{ $column }}</option>
                            @endif
                            @endforeach
                        </select>
                        <button type="button" class="btn btn-primary" id="btn-tampilkan">Tampilkan</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
<script>
    $(function() {
        $('#btn-tampilkan').on('click', function(){
            let val = $('#setting-column').val();
            const dt = $('#{{$tableId}}').DataTable();
            for(i=0;i<(dt.columns().header().length)-1;i++){
                let column = dt.column(i);
                if(jQuery.inArray( i.toString(), val ) != -1){
                    column.visible(true);
                }else{
                    column.visible(false);
                }
            }
        });
    })
</script>
