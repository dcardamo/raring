<div class="about-raring">
 <dl>
  <div>
   <dt>Your Garmin account, not your watch</dt>
   <dd>Sign in on Garmin’s own page. Raring reads the data in your Garmin account; it doesn’t connect directly to your watch.</dd>
  </div>
  <div>
   <dt>Keep Garmin Connect syncing</dt>
   <dd>Keep Garmin Connect installed and let it sync your watch. Raring can only show data that has reached Garmin.</dd>
  </div>
  <div>
   <dt>Your data stays with you</dt>
   <dd>Raring’s copy stays on your phone. Exports leave it only when you choose to share them. Nothing is sent to us. No analytics, telemetry, ads or separate Raring account.</dd>
  </div>
 </dl>
 <p class="connection-note">Raring is independent, not affiliated with Garmin. It uses an unofficial connection that Garmin can change or stop at any time. Available metrics depend on your watch.</p>
 <details>
  <summary>Privacy &amp; data details</summary>
  <div class="privacy-details">
   <p>Raring connects directly to Garmin to sign in and download your health data and activities, including GPS routes when available. Garmin’s services and sign-in page are covered by Garmin’s privacy policy.</p>
   <p>Raring does not read or store your Garmin password. Sign-in tokens are encrypted using Android Keystore. Downloaded data is stored in the app’s private storage, protected by Android, not by a separate Raring database encryption layer. Android backup and device transfer are disabled for the app.</p>
   <p>Settings → Export makes a ZIP of saved Garmin data and local settings, without sign-in tokens. It leaves your phone only when you choose a destination in the share sheet. The temporary ZIP is removed from the app’s cache afterwards.</p>
   <p>Street maps under workout routes come from OpenFreeMap, delivered through Cloudflare. Raring downloads each area once, usually 20–40 km across; they see your IP address and that area, never your route or health data. Turning street maps off in Settings stops downloads and removes downloaded maps. Map data © OpenStreetMap contributors, available under the Open Database License (openstreetmap.org/copyright), and © OpenMapTiles.</p>
   <p>Garmin journal reads are off until you allow them. Turning them off stops new reads and hides the journal. Previously downloaded logs remain on this device and in exports until local data is erased; they are never included in notifications.</p>
   <p>Android verifies your Garmin account before showing cached data. Reconnecting the same account preserves local history and targets. Signing into a different account clears the previous account’s cached data and local settings; export first if you need to keep those settings.</p>
   <p>Sign out in Settings to remove your saved sign-in and local Garmin history from Raring, or clear the app’s storage in Android. This does not delete anything from your Garmin account.</p>
   <p>Notifications are generated on your phone, not by a server. They may show health information on your lock screen; you can turn them off in Settings or change their visibility in Android.</p>
   <p>Raring is a fitness aid, not medical advice.</p>
  </div>
 </details>
</div>

<style>
 .about-raring{width:100%;text-align:left;font-size:14px;line-height:1.55}
 dl{display:grid;gap:16px;margin:0}
 dl>div{padding-left:14px;border-left:2px solid var(--ink)}
 dt{font-weight:650;color:var(--text);margin-bottom:4px}
 dd{margin:0;color:var(--text2)}
 .connection-note{color:var(--text2);font-size:12px;margin:18px 0 14px}
 details{border-top:1px solid var(--line);padding-top:12px}
 summary{color:var(--ink);cursor:pointer;min-height:44px;align-content:center}
 summary:focus-visible{outline:2px solid var(--ink);outline-offset:4px;border-radius:4px}
 .privacy-details{color:var(--text2)}
 .privacy-details p{margin:0 0 12px}
</style>
