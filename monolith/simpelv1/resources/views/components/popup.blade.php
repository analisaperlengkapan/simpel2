<!-- Vertically Centered -->
<div class="modal fade {{ $modalId }} zoomIn" tabindex="-1" role="dialog">
    <div class="modal-dialog modal-dialog-centered modal-lg">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Pemeberitahuan</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close">
                </button>
            </div>
            <div class="modal-body text-center p-5">
                <div id="carouselExampleControlsNoTouching" class="carousel carousel-dark slide" data-bs-touch="false"
                    data-bs-interval="false">
                    <div class="carousel-inner">
                        @foreach ($data as $item)
                            <div class="carousel-item {{ $loop->index == 0 ? 'active' : '' }}">
                                <h5>{{ $item['judul'] }}</h5>
                                {!! $item['isi'] !!}
                            </div>
                        @endforeach

                    </div>
                    @if (count($data) > 1)
                        <button class="carousel-control-prev" type="button"
                            data-bs-target="#carouselExampleControlsNoTouching" data-bs-slide="prev">
                            <span class="carousel-control-prev-icon" aria-hidden="true"></span>
                            <span class="visually-hidden">Previous</span>
                        </button>
                        <button class="carousel-control-next" type="button"
                            data-bs-target="#carouselExampleControlsNoTouching" data-bs-slide="next">
                            <span class="carousel-control-next-icon" aria-hidden="true"></span>
                            <span class="visually-hidden">Next</span>
                        </button>
                    @endif
                </div>
            </div>
            <div class="modal-footer">
                <a href="javascript:void(0);" class="btn btn-link link-success fw-medium" data-bs-dismiss="modal"><i
                        class="ri-close-line me-1 align-middle"></i> Tutup</a>
            </div>
        </div><!-- /.modal-content -->
    </div><!-- /.modal-dialog -->
</div>
<script>
    const modalId = `{{ $modalId }}`;
    $(function() {
        $('.' + modalId).modal('show');
    })
</script>
