# Logout Integration Testing Guide

## Prerequisites

1. **Backend Running**
   ```bash
   cd layanan/authenc
   cargo run
   # Should be running on http://localhost:8080
   ```

2. **Portal Running**
   ```bash
   cd antarmuka/portal
   trunk serve
   # Should be running on http://localhost:3000
   ```

## Manual Testing Steps

### Test 1: Basic Logout Flow

1. **Login**
   - Navigate to http://localhost:3000/login
   - Enter credentials and login
   - Verify you're redirected to dashboard

2. **Check SSO Cookie**
   - Open DevTools (F12)
   - Go to Application → Cookies → http://localhost:8080
   - Verify `AUTHENC_SSO` cookie exists
   -okie value

3. **Check localStorage**
   - In DevTools, go to Application → Local Storage → http://localhost:3000
   - Verify `user_session` and `auth_token` exist

4. **Logout**
   - Click logout button in navbar
   - Should redirect to `/logged-out` page
   - Verify you see "Anda Telah Keluar" message

5. **Verify Cleanup**
   - Check Application → Cookies → http://localhost:8080
   -` cookie should be deleted (or Max-Age=0)
   - Check Application → Local Storage
   - `user_session` and `auth_token` should be deleted

6. **Verify Backend Session**
   - Try to access protected endpoint with old token
   - Should get 401 Unauthorized

### Test 2: Cross-Tab Logout

1. **Open Two Tabs**
   - Open http://localhost:3000 in two browser tabs
   - Login in both tabs

2. **Logout from Tab 1**
   - Click logout in first tab
   - Should redirect to `/logged-out`

3. **Check Tab 2**
   - Switch to second tab
   - Should automatically logout (via storage event)
   - Session should be cleared

### Test 3: Redirect URI Validation

1. **Test Valid Redirect**
   - Logout normally
   - Should redirect to http://localhost:3000/logged-out

2. **Test Invalid Redirect** (Manual URL)
   - Navigate to: http://localhost:8080/v1/oidc/logout?post_logout_redirect_uri=https://evil.com
   - Should redirect to default Portal URL (not evil.com)

### Test 4: Network Failure Handling

1. **Stop Backend**
   - Stop authenc backend (Ctrl+C)

2. **Try Logout**
   - Click logout button
   - localStorage should still be cleared
   - UI should show logged out state
   - Network request will fail (expected)

3. **Restart Backend**
   - Start backend again
   - Login again
   - Logout should work normally

## Automated Testing

### Unit Tests

Run unit tests:
```bash
cd antarmuka/portal
cargo test --target wasm32-unknown-unknown
```

### Integration Tests

Run integration tests:
```bash
cd antarmuka/portal
wasm-pack test --headless --firefox
```

## Expected Results

### ✅ Success Criteria

- [ ] localStorage cleared immediately on logout
- [ ] Backend `/oidc/logout` endpoint called
- [ ] SSO cookie deleted (Max-Age=0)
- [ ] Redirect to `/logged-out` page
- [ ] Logged out page displays correctly
- [ ] Cross-tab logout works
- [ ] Invalid redirect URIs rejected
- [ ] Backend session invalidated

### ❌ Failure Indicators

- localStorage not cleared
- SSO cookie still valid after logout
- No redirect to logged out page
- Backend session still active
- Cross-tab logout not working
- Can access protected pages after logout

## Debugging

### Check Backend Logs

```bash
# In authenc terminal
# Look for logout events
grep "User logout completed" logs/authenc.log
grep "Published logout event" logs/authenc.log
```

### Check Browser Console

```javascript
// Check localStorage
console.log(localStorage.getItem('user_session'));
console.log(localStorage.getItem('auth_token'));

// Check cookies
console.log(document.cookie);
```

### Check Network Requests

1. Open DevTools → Network tab
2. Filter by "logout"
3. Verify request to `/v1/oidc/logout`
4. Check request includes credentials (cookies)
5. Check response status (should be 302 redirect)

## Common Issues

### Issue 1: CORS Error

**Symptom**: Console shows CORS error when calling logout endpoint

**Solution**:
- Check backend CORS configuration
- Ensure `allow_credentials(true)` is set
- Ensure Portal origin is in allowed origins

### Issue 2: Cookie Not Deleted

**Symptom**: AUTHENC_SSO cookie still exists after logout

**Solution**:
- Check cookie domain matches
- Verify credentials included in request
- Check backend cookie deletion logic

### Issue 3: Redirect Not Working

**Symptom**: Stays on same page after logout

**Solution**:
- Check backend redirect response
- Verify `post_logout_redirect_uri` parameter
- Check browser console for errors

### Issue 4: Cross-Tab Not Working

**Symptom**: Other tabs don't logout automatically

**Solution**:
- Check storage event listener
- Verify `logout_event` in localStorage
- Check browser storage event support

## Performance Metrics

### Target Metrics

- **Logout Latency**: < 500ms (from click to redirect)
- **localStorage Clear**: < 10ms
- **Backend Call**: < 200ms
- **Total Time**: < 1 second

### Measure Performance

```javascript
// In browser console
performance.mark('logout-start');
// Click logout
performance.mark('logout-end');
performance.measure('logout-duration', 'logout-start', 'logout-end');
console.log(performance.getEntriesByName('logout-duration'));
```

## Security Checklist

- [ ] Session invalidated on server
- [ ] SSO cookie deleted with proper flags
- [ ] localStorage cleared
- [ ] Refresh token revoked
- [ ] Logout event logged for audit
- [ ] Redirect URI validated
- [ ] No sensitive data in URL
- [ ] HTTPS in production

## Production Deployment Checklist

- [ ] Update AUTHENC_URL to production URL
- [ ] Update PORTAL_URL to production URL
- [ ] Test in staging environment
- [ ] Verify HTTPS certificates
- [ ] Check CORS configuration
- [ ] Monitor logout success rate
- [ ] Set up alerts for logout failures
- [ ] Document rollback procedure

## Rollback Procedure

If issues occur in production:

1. **Immediate Rollback**
   ```bash
   git revert <commit-hash>
   git push
   # Redeploy portal
   ```

2. **Verify Old Version**
   - Test logout still works (localStorage only)
   - Check no breaking changes

3. **Investigate Issue**
   - Check logs
   - Review error reports
   - Fix and redeploy

## Support

For issues or questions:
- Check logs: `logs/authenc.log` and browser console
- Review documentation: `FRONTEND_BACKEND_LOGOUT_INTEGRATION.md`
- Contact: dev@kejaksaan.go.id

