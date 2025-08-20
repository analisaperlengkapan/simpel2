// File: internal/integrasi/mysimkari/sync_all.go
package mysimkari

import (
	"context"
	"simpelv2/layanan-integrasi/internal/db/generated"
)

func Sync(ctx context.Context, q *generated.Queries) error {
	if err := SyncSatker(ctx, q); err != nil {
		return err
	}
	if err := SyncPegawai(ctx, q); err != nil {
		return err
	}
	return nil
}
