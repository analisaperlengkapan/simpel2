@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    
                </div>
                <div class="card-body">
                <form action="/suport/survey" method="POST" class="ajaxForm" >
                       
                        <div class="row">
                            <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="pertanyaan 1" class="form-label">Pertanyaan Pertama*</label>
                                    <div class="form-check">
                                        <input class="form-check-input" type="radio" name="pertanyaan1" id="pertanyaan11">
                                        <label class="form-check-label" for="pertanyaan11">
                                            Ya
                                        </label>
                                    </div>
                                    <div class="form-check">
                                        <input class="form-check-input" type="radio" name="pertanyaan1" id="pertanyaan12">
                                        <label class="form-check-label" for="pertanyaan12">
                                            Tidak
                                        </label>
                                    </div>
                                    
                                </div>
                            </div>
                            
                        </div>
                        <div class="row">
                        <div class=" col-lg-6">
                                <div class="mb-3">
                                    <label for="pertanyaan 2" class="form-label">Pertanyaan Ke Dua*</label>
                                    <div class="form-check">
                                        <input class="form-check-input" type="radio" name="pertanyaan2" id="pertanyaan21">
                                        <label class="form-check-label" for="pertanyaan21">
                                           Setuju
                                        </label>
                                    </div>
                                    <div class="form-check">
                                        <input class="form-check-input" type="radio" name="pertanyaan2" id="pertanyaan22">
                                        <label class="form-check-label" for="pertanyaan22">
                                            Tidak Setuju
                                        </label>
                                    </div>
                                    
                                </div>
                            </div> 
                        </div>
                        <div class="row">
                            <div class="col-lg-12">
                                <div class="hstack gap-2 justify-content-end">
                                    <a href="{{ url('suport/faq') }}" class="btn btn-outline-primary">Kembali</a>
                                    
                                    <button type="submit" class="btn btn-primary">
                                         Simpan
                                    </button>
                                    
                                </div>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
        <!--end col-->
    </div>
<style>
    #{{ $tableId }} thead th {
        background-color: #405189;
        color: #ffffff;
        text-align: center;
        text-transform: uppercase;
    }
</style>
@endsection

@section('js')
<script>
    
</script>
@endsection
