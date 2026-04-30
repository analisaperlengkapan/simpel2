<div id="formModal" class="modal fade zoomIn" tabindex="-1">
    <div class="modal-dialog modal-xl modal-dialog-centered">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Pilih Asset</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
                <div class="row">
                    <table id="{{ $tableId }}" class="display table table-bordered dt-responsive my-dt"
                        style="width:100%">
                        <thead>
                            <tr>
                                <th></th>
                                @foreach ($columns as $index => $column)
                                    <th>
                                        {{ $column }}
                                    </th>
                                @endforeach
                            </tr>
                        </thead>

                    </table>

                </div>
            </div>
            <div class="modal-footer">
                <button type="button" class="btn btn-light" data-bs-dismiss="modal" onc>Tutup</button>
                <button type="button" id="assetAdd" class="btn btn-primary ">Tambah</button>
            </div>

        </div>
    </div>
</div>
<script></script>
<script>
    const tableId = `{{ $tableId }}`;
    const defColumn = @json($defColumns);
    const assets = @json($assetSelections);

    $(function() {
        const selectedAssetTbl = $('#tableSelectedAsset').DataTable();

        function appendRow({
            no,
            id,
            data,
        }) {
            let elm = `
                <tr id="assetRow-${id}">
                    <td>${data.nm_aset}</td>
                    <td>${data.kode_barang}</td>
                    <td>${data.nm_barang}</td>
                    <td>${data.nup}</td>
                    <td>${int2money(data.nilai_perolehan)}</td>
                    <td>
                        <div class="d-flex justify-content-center gap-2">
                            <button type="button" class="btn btn-danger waves-effect waves-light deleteRow" >
                                <i class="ri-delete-bin-5-fill"></i>
                            </button>
                        </div>
                        <div>`;

            elm += `<input type="hidden" class="selectedAssets" name="vw_aset_psp_ids[]" value="${id}">`
            elm += `
                            </div>
                    </td>
                </tr>`;

            selectedAssetTbl.row.add($(elm)).draw().node();
            $('#formModal').modal('hide')
        }
        $(document).on('click', '.deleteRow', function() {
            selectedAssetTbl.row($(this).parents('tr')).remove().draw(false);
            refreshPilihanAsset();
        })


        function refreshPilihanAsset() {
            const selecteds = []
            $('.selectedAssets').each((i, elm) => selecteds.push($(elm).val()));
            const filteredSource = assets.filter(asset => !selecteds.includes(asset.id))
            dt.clear().rows.add(filteredSource).draw();
        }

        const dt = $('#' + tableId).DataTable({
            responsive: true,
            language: {
                url: `{{ url('/assets/js/datatable_bahasa.json') }}`,
            },
            data: assets,
            columnDefs: [{
                orderable: false,
                checkboxes: {
                    selectRow: true
                },
                targets: 0
            }],
            select: {
                style: 'multi',
                selector: 'td:first-child'
            },
            columns: [{
                    data: 'id',
                    className: 'select-checkbox',
                    render: () => ''
                }, {
                    data: 'nm_aset',
                },
                {
                    data: 'kode_barang',
                    className: 'text-center'
                },
                {
                    data: 'nm_barang',
                },
                {
                    data: 'nup',
                    className: 'text-center'
                },
                {
                    data: 'nilai_perolehan',
                    className: 'text-end',
                    render: (data) => int2money(data)
                },
            ]

        });
        $('#assetAdd').on('click', function() {
            const rows_selected = dt.column(0).checkboxes.selected();
            const currentSelected = [];
            $.each(rows_selected, function(index, rowId) {
                currentSelected.push(rowId);
            });

            $.post(`{{ $controller }}/getSelectedAsset`, {
                whereIn: currentSelected
            }, function(res) {
                if (res.length < 1) return;
                res.map((aset, i) => appendRow({
                    data: aset,
                    id: aset.id,
                    no: i + 1
                }))
                refreshPilihanAsset();
            })
        });
    });
</script>
