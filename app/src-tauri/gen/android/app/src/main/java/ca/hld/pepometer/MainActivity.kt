package ca.hld.pepometer

import android.graphics.Color
import android.os.Bundle
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)
    window.decorView.setBackgroundColor(Color.parseColor("#0B0F1A"))
    // Edge-to-edge (forced from targetSdk 35) gives the WebView no safe-area
    // insets, so pad the content view by the system bars and the keyboard.
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { v, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
      val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
      v.setPadding(bars.left, bars.top, bars.right, maxOf(bars.bottom, ime.bottom))
      WindowInsetsCompat.CONSUMED
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    // Tauri installs its history callback during plugin setup. Register after
    // that, when the WebView exists, so it cannot override our page hierarchy.
    onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        if (webView.url?.startsWith("http://tauri.localhost/") == false) {
          webView.loadUrl("http://tauri.localhost/")
        } else {
          webView.evaluateJavascript("window.__pepometerBack?.() === true") { handled ->
            if (handled != "true") moveTaskToBack(true)
          }
        }
      }
    })
  }

  override fun onResume() {
    super.onResume()
    // The first resume may precede WebView creation; the app loads its initial data itself.
    val webview = findWebView(findViewById(android.R.id.content))
    if (webview?.url?.startsWith("http://tauri.localhost/") == true) {
      webview.evaluateJavascript("window.dispatchEvent(new Event('pepometer:resume'))", null)
    }
  }

  private fun findWebView(view: View): WebView? {
    if (view is WebView) return view
    if (view is ViewGroup) for (i in 0 until view.childCount) {
      findWebView(view.getChildAt(i))?.let { return it }
    }
    return null
  }
}
