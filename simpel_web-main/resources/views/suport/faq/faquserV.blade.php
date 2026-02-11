@extends('layout.main')
@section('content')
@include('components.breadcums', $breadcums)
    <div class="row">
        <div class="col-lg-12">
            <div class="card">
                <div class="card-header">
                    <div class="d-flex align-items-center">
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-0">FAQ</h5>
                        </div>
                        <div class="flex-shrink-0">

                        </div>
                    </div>
                    <!-- <div class="row mt-3">
                        @include('components.dtSearchBox', [
                            'tableId' => $tableId,
                            'filterBy' => [
                                ['text' => 'Pertanyaan', 'value' => 'pertanyaan'],
                                ['text' => 'Jawaban', 'value' => 'jawaban'],
                            ],
                        ])
                    </div> -->
                </div>
                <div class="card-body">
                <div class="accordion accordion-flush" id="accordionFlushExample">
                    @foreach($faq as $index => $item)
                    <div class="accordion-item">
                        <h2 class="accordion-header">
                        <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#flush-collapse{{$index}}" aria-expanded="false" aria-controls="flush-collapse{{$index}}">
                            {{$item['pertanyaan']}}
                        </button>
                        </h2>
                        <div id="flush-collapse{{$index}}" class="accordion-collapse collapse" data-bs-parent="#accordionFlushExample">
                        <div class="accordion-body">{{$item['jawaban']}}</div>
                        </div>
                    </div>
                    @endforeach
                    </div>
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
