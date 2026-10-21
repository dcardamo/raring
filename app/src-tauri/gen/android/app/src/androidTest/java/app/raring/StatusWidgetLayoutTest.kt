package app.raring

import android.content.Context
import android.content.res.Configuration
import android.graphics.Rect
import android.util.Xml
import android.view.View
import android.view.ViewGroup
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
    private val waitingFocus = "Waiting for last night's sleep"

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

    private fun snapshot(missing: Boolean) = if (missing) StatusSnapshot("As of unavailable", waitingFocus, "today",
        StatusMetric(null, "neutral"), StatusMetric(null, "neutral"), StatusMetric(null, "neutral"), StatusMetric(null, "neutral"))
    else StatusSnapshot(eveningAsOf, eveningFocus, "sleep", StatusMetric(87.0, "good"), StatusMetric(100.0, "good"),
        StatusMetric(100.0, "good"), StatusMetric(100.0, "neutral"))

    private fun rendered(context: Context, medium: Boolean, missing: Boolean): Pair<View, Size> {
        val size = providerSize(context, if (medium) R.xml.status_widget_medium else R.xml.status_widget_small)
        val root = StatusSurfaces.widgetViews(context, medium, snapshot(missing)).apply(context, null)
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

    @Test fun remoteWidgetsFitAtProviderMinimumForLargeFontsAndMissingReadings() {
        for (fontScale in listOf(1f, 1.3f, 1.5f)) {
            val config = Configuration(app.resources.configuration).apply { this.fontScale = fontScale }
            val context = app.createConfigurationContext(config)
            for (medium in listOf(false, true)) for (missing in listOf(false, true)) {
                val (root, size) = rendered(context, medium, missing)
                val image = android.graphics.Bitmap.createBitmap(root.width, root.height, android.graphics.Bitmap.Config.ARGB_8888)
                root.draw(android.graphics.Canvas(image))
                val name = "widget-${if(medium) "medium" else "small"}-$fontScale-${if(missing) "missing" else "values"}.png"
                java.io.File(app.filesDir, name).outputStream().use { image.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it) }
                image.recycle()
                assertEquals(size.width, root.width)
                assertEquals(size.height, root.height)
                val caption = textViews(root).single { it.text.toString() == "Readiness" }
                checkText(root, caption, "Readiness")
                assertEquals("the RemoteViews must actually use the requested font scale",
                    android.util.TypedValue.applyDimension(android.util.TypedValue.COMPLEX_UNIT_SP, 10f, context.resources.displayMetrics),
                    caption.textSize, 0.5f)
                val asOf = root.findViewById<TextView>(R.id.status_asof)
                checkText(root, asOf, if (missing) "As of unavailable" else eveningAsOf)
                // A missing reading is a dash, never a blank or a zero.
                val value = if (missing) "\u2014" else "100"
                checkText(root, root.findViewById(R.id.status_readiness), if (missing) "\u2014" else "87")
                checkInside(root, root.findViewById(R.id.status_gauge), "gauge")
                val tiles = (listOf(R.id.status_tile_readiness) + if (medium) listOf(R.id.status_tile_sleep,
                    R.id.status_tile_battery, R.id.status_tile_stress) else emptyList()).map { root.findViewById<View>(it) }
                tiles.forEachIndexed { i, tile -> checkInside(root, tile, "tile $i tap region") }
                for (i in tiles.indices) for (j in i + 1 until tiles.size) {
                    assertTrue("tile tap regions overlap", !Rect.intersects(boundsInRoot(tiles[i], root), boundsInRoot(tiles[j], root)))
                }
                if (medium) {
                    for (label in listOf("Sleep", "Body Battery", "Stress")) {
                        checkText(root, textViews(root).single { it.text.toString() == label }, label)
                    }
                    for (id in listOf(R.id.status_sleep, R.id.status_battery, R.id.status_stress)) {
                        checkText(root, root.findViewById(id), value)
                    }
                } else {
                    checkText(root, root.findViewById(R.id.status_focus), if (missing) waitingFocus else eveningFocus)
                }
            }
        }
    }
}
