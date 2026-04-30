    <div class="col-lg-10 mx-auto">
        <div class="input-group">
            @if(isset($filterBy))
            <button class="btn btn-outline-secondary btn-border dropdown-toggle" id="dt-filter-by" type="button" data-bs-toggle="dropdown" aria-expanded="false" data-filterby="">Pencarian</button>
            <ul class="dropdown-menu">
                @foreach ($filterBy as $filter)
                <li><a class="dropdown-item dt-filter-by-option" href="javascript:;" data-value="{{$filter['value']}}">{{$filter['text']}}</a></li>
                @endforeach
            </ul>
            @endif
            <input type="text" class="form-control" aria-label="Pencarian...." id="dt-search-input" />
            <button class="input-group-text btn-dark btn" id="dt-search-btn" type="button">
                <span class="">
                    <i class="ri-search-line align-bottom me-1"></i>
                    Cari
                </span>
            </button>
        </div>
    </div>

    <script>
        $(function() {
            const tableId = '#{{$tableId}}';
            $('#dt-search-input').on('keydown', function(e) {
                if (e.keyCode == 13) $('#dt-search-btn').trigger('click');
            })

            $('#dt-search-btn').on('click', function() {
                const searchValue = $('#dt-search-input').val();
                const dt = $(tableId).DataTable();
                dt.search(searchValue).draw();
            });

            $('.dt-filter-by-option').on('click', function() {
                const categoryVal = $(this).data('value');
                const categoryText = $(this).text();
                $('#dt-filter-by').data('filterby', categoryVal).text(categoryText);
            })
        })
    </script>