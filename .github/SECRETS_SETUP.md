# GitHub Secrets Setup Guide

This guide explains how to set up all required secrets for automated builds.

## Required Secrets

### 1. KEYSTORE_BASE64

Your Android keystore file encoded in base64.

**How to generate:**

```bash
# Navigate to your keystore location
cd android/app

# Encode keystore to base64 (single line)
base64 -i release.keystore | tr -d '\n' > keystore_base64.txt

# Copy the content of keystore_base64.txt
cat keystore_base64.txt

# Clean up
rm keystore_base64.txt
```

**Add to GitHub:**
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `KEYSTORE_BASE64`
4. Value: Paste the base64 string
5. Click "Add secret"

---

### 2. KEYSTORE_PASSWORD

The password you used when creating the keystore.

**Add to GitHub:**
1. Settings → Secrets and variables → Actions
2. New repository secret
3. Name: `KEYSTORE_PASSWORD`
4. Value: Your keystore password
5. Add secret

---

### 3. KEY_ALIAS

The alias name of your signing key.

**How to find:**
```bash
# List aliases in your keystore
keytool -list -v -keystore release.keystore
# Enter keystore password when prompted
# Look for "Alias name:" in the output
```

**Add to GitHub:**
1. Settings → Secrets and variables → Actions
2. New repository secret
3. Name: `KEY_ALIAS`
4. Value: Your key alias (e.g., "kira" or "release")
5. Add secret

---

### 4. KEY_PASSWORD

The password for your specific key alias.

**Add to GitHub:**
1. Settings → Secrets and variables → Actions
2. New repository secret
3. Name: `KEY_PASSWORD`
4. Value: Your key password
5. Add secret

---

## Optional: Telegram Notifications

### 5. TELEGRAM_BOT_TOKEN

Token for your Telegram bot to send APK notifications.

**How to get:**
1. Open Telegram and search for `@BotFather`
2. Send `/newbot` command
3. Follow instructions to create a bot
4. Copy the token (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

**Add to GitHub:**
1. Settings → Secrets and variables → Actions
2. New repository secret
3. Name: `TELEGRAM_BOT_TOKEN`
4. Value: Your bot token
5. Add secret

---

### 6. TELEGRAM_CHAT_ID

The chat ID of your Telegram group/channel.

**How to get:**
1. Add your bot to your group/channel
2. Send a message in the group
3. Visit: `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
4. Look for `"chat":{"id":-1234567890}` in the response
5. Copy the chat ID (including the minus sign if present)

**Alternative method:**
```bash
# Send a message to your bot/group, then run:
curl https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates | jq '.result[0].message.chat.id'
```

**Add to GitHub:**
1. Settings → Secrets and variables → Actions
2. New repository secret
3. Name: `TELEGRAM_CHAT_ID`
4. Value: Your chat ID (e.g., `-1001234567890`)
5. Add secret

---

## Creating a New Keystore (If Needed)

If you don't have a keystore yet:

```bash
# Navigate to android/app directory
cd android/app

# Generate new keystore
keytool -genkey -v -keystore release.keystore \
  -alias kira \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000

# You'll be prompted for:
# - Keystore password (remember this!)
# - Key password (remember this!)
# - Your name, organization, etc.
```

**Important:**
- Keep your keystore file safe and backed up
- Never commit keystore to git
- Store passwords securely (password manager recommended)
- If you lose the keystore, you cannot update your app on Play Store

---

## Verification

After adding all secrets, verify they're set:

1. Go to Settings → Secrets and variables → Actions
2. You should see:
   - ✅ KEYSTORE_BASE64
   - ✅ KEYSTORE_PASSWORD
   - ✅ KEY_ALIAS
   - ✅ KEY_PASSWORD
   - ✅ TELEGRAM_BOT_TOKEN (optional)
   - ✅ TELEGRAM_CHAT_ID (optional)

---

## Testing

Test your setup by:

1. Creating a test tag:
   ```bash
   git tag v0.0.1-test
   git push origin v0.0.1-test
   ```

2. Check GitHub Actions tab for build status

3. If Android build succeeds, you should receive APK in Telegram (if configured)

4. Delete test tag after verification:
   ```bash
   git tag -d v0.0.1-test
   git push origin :refs/tags/v0.0.1-test
   ```

---

## Troubleshooting

### "Keystore was tampered with, or password was incorrect"
- Check KEYSTORE_PASSWORD is correct
- Verify KEYSTORE_BASE64 was encoded correctly

### "Cannot recover key"
- Check KEY_PASSWORD is correct
- Verify KEY_ALIAS matches your keystore

### Telegram not sending
- Verify bot token is correct
- Ensure bot is added to the group
- Check chat ID includes minus sign if it's a group

### Build fails on signing
- Ensure all 4 keystore secrets are set
- Check keystore file is not corrupted
- Verify base64 encoding has no line breaks
