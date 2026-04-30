<br />
@if (str_contains(strtolower($column), 'tanggal') || str_contains(strtolower($column), 'tgl'))
@include('components.datepicker',[
    'index'=>$index,
    'name'=>'',
    'className'=>'column-filter'
    ]
)
@elseif ($column == 'combo_satker_all')
<select class="column-filter form-control" data-column="{{ $index }}" id="combo_satker_all">
    <option value="" selected>Pilih Satker</option>
    {!! $satkerAllOptions !!}
</select>   
@elseif ($column == 'kategori_bast')
<select class="column-filter form-control" data-column="{{ $index }}">
    <option value="">--Pilih--</option>
</select>
@elseif ($column == 'nilai_bast')
<select class="column-filter form-control" data-column="{{ $index }}">
    <option value="">--Pilih--</option>
    <option value="seratus">S.D 100 Juta</option>
    <option value="seratus_lebih">Lebih Dari 100 Juta</option>
</select>
@elseif ($column == 'combo_nilai_kontrak')
<select class="column-filter form-control" data-column="{{ $index }}">
    <option value="">--Pilih--</option>
    <option value="duaratus">S.D 200 Juta</option>
    <option value="duaratus_lebih">Lebih Dari 200 Juta</option>
</select>
@else
<input type="text" class="column-filter form-control" data-column="{{ $index }}" />
@endif
