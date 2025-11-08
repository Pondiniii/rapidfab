# S3 Lifecycle Configuration

## Hetzner S3 Lifecycle Rule (7-day TTL for Anonymous Uploads)

To automatically delete anonymous uploads after 7 days, configure S3 lifecycle policy:

### Using AWS CLI:

1. Create `lifecycle.json`:
```json
{
  "Rules": [
    {
      "Id": "delete-anon-uploads",
      "Status": "Enabled",
      "Prefix": "anon/",
      "Expiration": {
        "Days": 7
      }
    }
  ]
}
```

2. Apply to bucket:
```bash
aws s3api put-bucket-lifecycle-configuration \
  --bucket rapidfab \
  --endpoint-url https://fsn1.your-objectstorage.com \
  --lifecycle-configuration file://lifecycle.json
```

### Verification:
```bash
aws s3api get-bucket-lifecycle-configuration \
  --bucket rapidfab \
  --endpoint-url https://fsn1.your-objectstorage.com
```

### Alternative: Web Console
- Log into Hetzner Object Storage console
- Select bucket `rapidfab`
- Navigate to "Lifecycle" tab
- Add rule: Prefix=`anon/`, Expiration=7 days

## Folder Structure

- `anon/{session_id}/{file_id}.ext` - Deleted after 7 days
- `users/{user_id}/{file_id}.ext` - Permanent (user-managed)
