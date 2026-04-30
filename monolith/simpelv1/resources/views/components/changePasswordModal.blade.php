<form action="/auth/changePassword" method="POST" class="ajaxForm">
    <input type="hidden" name="user_id" value="{{isset($user_id) ? $user_id : session('userData.id')}}">
    @csrf
    <div id="changePasswordModal" class="modal fade zoomIn" tabindex="-1" aria-labelledby="zoomInModalLabel" aria-hidden="true" style="display: none;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title" id="zoomInModalLabel">Ubah Password</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <div class="row">
                        <div class=" col-lg-12">
                            <div class="mb-3">
                                <label for="password_old" class="form-label">Password Lama</label>
                                <input type="password" class="form-control" id="password_old" name="password_old" required placeholder="Password Lama">
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class=" col-lg-12">
                            <div class="mb-3">
                                <label for="password_new" class="form-label">Password Baru</label>
                                <input type="password" class="form-control" id="password_new" name="password_new" required placeholder="Password Baru">
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-light" data-bs-dismiss="modal">Tutup</button>
                    <button type="submit" class="btn btn-primary ">Ubah</button>
                </div>
            </div>
        </div>
    </div>
</form>
