package app.raring

import android.content.Context
import android.content.res.Configuration
import android.graphics.Rect
import android.util.Xml
import android.view.View
import android.view.ViewGroup
import android.widget.RemoteViews
import android.widget.TextView
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.xmlpull.v1.XmlPullParser

@RunWith(AndroidJUnit4::class)
class StatusWidgetLayoutTest {
    private val app = ApplicationProvider.getApplicationContext<Context>()
    private val eveningFocus = "Wind down at 22:00 tonight"
    private val eveningAsOf = "As of 28 Sep, 23:59"

    private data class Size(val width: Int, val height: Int)

    // Resolve the actual provider declaration rather than duplicating its dimensions in this test.
    private fun providerSize(context: Context, xmlId: Int): Size {
        val parser = context.resources.getXml(xmlId)
        try {
            while (parser.eventType != XmlPullParser.START_TAG) parser.next()
            val attributes = context.resources.obtainAttributes(
                Xml.asAttributeSet(parser),
                intArrayOf(android.R.attr.minWidth, android.R.attr.minHeight,
                    android.R.attr.minResizeWidth, android.R.attr.minResizeHeight)
            )
            try {
                val width = attributes.getDimensionPixelSize(0, 0)
                val height = attributes.getDimensionPixelSize(1, 0)
                assertTrue("provider must declare positive dimensions", width > 0 && height > 0)
                assertEquals("horizontal resize must not shrink below tested width", width,
                    attributes.getDimensionPixelSize(2, 0))
                assertEquals("vertical resize must not shrink below tested height", height,
                    attributes.getDimensionPixelSize(3, 0))
                return Size(width, height)
            } finally { attributes.recycle() }
        } finally { parser.close() }
    }

    private fun rendered(context: Context, medium: Boolean, missing: Boolean): Pair<View, Size> {
        val size = providerSize(context, if (medium) R.xml.status_widget_medium else R.xml.status_widget_small)
        val remote = RemoteViews(context.packageName,
            if (medium) R.layout.status_widget_medium else R.layout.status_widget_small)
        remote.setImageViewBitmap(R.id.status_gauge,
            StatusSurfaces.gauge(StatusMetric(if (missing) null else 87.0, "good"),
                context.resources.displayMetrics.density))
        remote.setTextViewText(R.id.status_readiness, if (missing) "" else "87")
        remote.setTextViewText(R.id.status_asof, if (missing) "As of unavailable" else eveningAsOf)
        if (medium) {
            remote.setTextViewText(R.id.status_sleep, if (missing) "" else "100")
            remote.setTextViewText(R.id.status_battery, if (missing) "" else "100")
            remote.setTextViewText(R.id.status_stress, if (missing) "" else "100")
        } else {
            remote.setTextViewText(R.id.status_focus, if (missing) "" else eveningFocus)
        }
        val root = remote.apply(context, null)
        root.measure(View.MeasureSpec.makeMeasureSpec(size.width, View.MeasureSpec.EXACTLY),
            View.MeasureSpec.makeMeasureSpec(size.height, View.MeasureSpec.EXACTLY))
        root.layout(0, 0, size.width, size.height)
        return root to size
    }

    private fun boundsInRoot(view: View, root: View): Rect {
        val rect = Rect(0, 0, view.width, view.height)
        (root as ViewGroup).offsetDescendantRectToMyCoords(view, rect)
        return rect
    }

    private fun checkInside(root: View, child: View, name: String) {
        val rect = boundsInRoot(child, root)
        assertTrue("$name has no touch/readable area: $rect", rect.width() > 0 && rect.height() > 0)
        assertTrue("$name clipped by provider bounds: $rect in ${root.width}x${root.height}",
            rect.left >= 0 && rect.top >= 0 && rect.right <= root.width && rect.bottom <= root.height)
        var ancestor = child.parent
        while (ancestor is View && ancestor !== root) {
            val parentBounds = boundsInRoot(ancestor, root)
            assertTrue("$name clipped by its parent: $rect vs $parentBounds", parentBounds.contains(rect))
            ancestor = (ancestor as View).parent
        }
    }

    private fun textViews(view: View): List<TextView> = when (view) {
        is TextView -> listOf(view)
        is ViewGroup -> (0 until view.childCount).flatMap { textViews(view.getChildAt(it)) }
        else -> emptyList()
    }

    private fun checkText(root: View, text: TextView, expected: String) {
        assertEquals(expected, text.text.toString())
        if (expected.isEmpty()) return // blank numeric wrap_content views need no text width; the tile remains tappable
        checkInside(root, text, "text '$expected'")
        val layout = text.layout
        assertNotNull("text '$expected' has no layout", layout)
        layout!!
        assertTrue("text '$expected' lost its last characters", layout.getLineEnd(layout.lineCount - 1) == expected.length)
        val contentWidth = text.width - text.compoundPaddingLeft - text.compoundPaddingRight
        val contentHeight = text.height - text.totalPaddingTop - text.totalPaddingBottom
        assertTrue("text '$expected' exceeds available height: ${layout.height} > $contentHeight",
            layout.height <= contentHeight + 1)
        for (line in 0 until layout.lineCount) {
            assertEquals("text '$expected' ellipsized on line $line", 0, layout.getEllipsisCount(line))
            assertTrue("text '$expected' overflows width on line $line",
                layout.getLineRight(line) - layout.getLineLeft(line) <= contentWidth + 1)
        }
    }

    @Test fun expandedNotificationLabelsFitAtLargeFont() {
        val config = Configuration(app.resources.configuration).apply { fontScale = 1.5f }
        val context = app.createConfigurationContext(config)
        val remote = RemoteViews(context.packageName, R.layout.status_notification_expanded)
        remote.setTextViewText(R.id.status_asof, eveningAsOf)
        val root = remote.apply(context, null)
        val width = (300 * context.resources.displayMetrics.density).toInt()
        root.measure(View.MeasureSpec.makeMeasureSpec(width, View.MeasureSpec.EXACTLY),
            View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED))
        root.layout(0, 0, root.measuredWidth, root.measuredHeight)
        for (label in listOf("Body Battery", "Stress", eveningAsOf)) {
            checkText(root, textViews(root).single { it.text.toString() == label }, label)
        }
    }

    @Test fun remoteWidgetsFitAtProviderMinimumForLargeFontsAndMissingReadings() {
        for (fontScale in listOf(1f, 1.3f, 1.5f)) {
            val config = Configuration(app.resources.configuration).apply { this.fontScale = fontScale }
            val context = app.createConfigurationContext(config)
            for (medium in listOf(false, true)) for (missing in listOf(false, true)) {
                val (root, size) = rendered(context, medium, missing)
                assertEquals(size.width, root.width)
                assertEquals(size.height, root.height)
                val title = textViews(root).single { it.text.toString() == "Raring" }
                checkText(root, title, "Raring")
                assertEquals("the RemoteViews must actually use the requested font scale",
                    android.util.TypedValue.applyDimension(android.util.TypedValue.COMPLEX_UNIT_SP, 12f, context.resources.displayMetrics),
                    title.textSize, 0.5f)
                val asOf = root.findViewById<TextView>(R.id.status_asof)
                checkText(root, asOf, if (missing) "As of unavailable" else eveningAsOf)
                val readiness = root.findViewById<TextView>(R.id.status_readiness)
                checkText(root, readiness, if (missing) "" else "87")
                val gauge = root.findViewById<View>(R.id.status_gauge)
                checkInside(root, gauge, "gauge tap region")
                for (label in listOf("Readiness") + if (medium) listOf("Sleep", "Body Battery", "Stress") else emptyList()) {
                    checkText(root, textViews(root).single { it.text.toString() == label }, label)
                }
                if (medium) {
                    val tiles = listOf(R.id.status_tile_readiness, R.id.status_tile_sleep,
                        R.id.status_tile_battery, R.id.status_tile_stress)
                        .map { root.findViewById<View>(it) }
                    tiles.forEachIndexed { i, tile -> checkInside(root, tile, "tile $i tap region") }
                    for (i in 0 until tiles.size - 1) {
                        assertTrue("tile tap regions overlap", boundsInRoot(tiles[i], root).right <=
                            boundsInRoot(tiles[i + 1], root).left)
                    }
                    for (id in listOf(R.id.status_sleep, R.id.status_battery, R.id.status_stress)) {
                        checkText(root, root.findViewById(id), if (missing) "" else "100")
                    }
                } else {
                    checkInside(root, root.findViewById(R.id.status_widget_root), "widget tap region")
                    checkInside(root, readiness, "readiness tap region")
                    checkText(root, root.findViewById(R.id.status_focus), if (missing) "" else eveningFocus)
                }
                val image = android.graphics.Bitmap.createBitmap(root.width, root.height, android.graphics.Bitmap.Config.ARGB_8888)
                root.draw(android.graphics.Canvas(image))
                val name = "widget-${if(medium) "medium" else "small"}-$fontScale-${if(missing) "missing" else "values"}.png"
                java.io.File(app.filesDir, name).outputStream().use { image.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it) }
                image.recycle()
            }
        }
    }
}
