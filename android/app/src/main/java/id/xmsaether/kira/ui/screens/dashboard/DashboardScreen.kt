package id.xmsaether.kira.ui.screens.dashboard

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun DashboardScreen(vm: DashboardViewModel = viewModel()) {
    val state by vm.uiState.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text(
            text = "KIRA",
            style = MaterialTheme.typography.headlineLarge.copy(fontWeight = FontWeight.Bold),
            color = MaterialTheme.colorScheme.onSurface
        )

        if (state.isLoading) {
            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        }

        state.error?.let {
            Card(
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer),
                shape = RoundedCornerShape(16.dp)
            ) {
                Text(text = it, modifier = Modifier.padding(16.dp), color = MaterialTheme.colorScheme.onErrorContainer)
            }
        }

        state.deviceInfo?.let { info -> DeviceInfoCard(info) }

        SparkLineCard(
            title = "CPU",
            value = "${state.currentCpu}%",
            subtitle = state.uptimeStr,
            data = state.cpuHistory,
            lineColor = Color(0xFF4ADE80),
            fillColor = Color(0xFF4ADE80).copy(alpha = 0.15f)
        )

        SparkLineCard(
            title = "Memory",
            value = "${state.currentMem}%",
            subtitle = "${state.memUsedMb}MB / ${state.memTotalMb}MB",
            data = state.memHistory,
            lineColor = Color(0xFFA78BFA),
            fillColor = Color(0xFFA78BFA).copy(alpha = 0.15f)
        )

        SparkLineCard(
            title = "FPS",
            value = "${state.currentFps}",
            subtitle = "SurfaceFlinger",
            data = state.fpsHistory,
            lineColor = Color(0xFFF97316),
            fillColor = Color(0xFFF97316).copy(alpha = 0.15f)
        )

        state.performance?.battery?.let { bat ->
            Card(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(20.dp),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh)
            ) {
                Row(
                    modifier = Modifier.padding(16.dp).fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        Icon(Icons.Filled.Favorite, contentDescription = null, tint = MaterialTheme.colorScheme.primary, modifier = Modifier.size(20.dp))
                        Column {
                            Text("Battery", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text("${bat.level}%", style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold))
                        }
                    }
                    Text("${bat.temperature / 10.0}°C • ${bat.voltage}mV", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }

        if (state.coreHistories.isNotEmpty()) {
            CpuCoresSection(state)
        }
    }
}

@Composable
private fun DeviceInfoCard(info: id.xmsaether.kira.data.model.DeviceInfo) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(24.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer)
    ) {
        Column(modifier = Modifier.padding(20.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier.size(48.dp).clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.primary),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(Icons.Filled.Phone, contentDescription = null, tint = MaterialTheme.colorScheme.onPrimary, modifier = Modifier.size(24.dp))
                }
                Spacer(modifier = Modifier.width(14.dp))
                Column {
                    Text(
                        text = info.model.ifEmpty { "Unknown" },
                        style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold),
                        color = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                    Text(
                        text = "${info.manufacturer} • Android ${info.androidVersion}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f)
                    )
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
            HorizontalDivider(color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.12f))
            Spacer(modifier = Modifier.height(12.dp))
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceEvenly) {
                InfoChip("ABI", info.abi.ifEmpty { "-" })
                InfoChip("Slot", info.slot.ifEmpty { "-" })
                InfoChip("Refresh", "${info.refreshRate}Hz")
            }
        }
    }
}

@Composable
private fun InfoChip(label: String, value: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(label, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.5f))
        Text(value, style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold), color = MaterialTheme.colorScheme.onPrimaryContainer, maxLines = 1)
    }
}

@Composable
private fun SparkLineCard(
    title: String,
    value: String,
    subtitle: String,
    data: List<Float>,
    lineColor: Color,
    fillColor: Color
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(20.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainer)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Column {
                    Text(title, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text(value, style = MaterialTheme.typography.headlineSmall.copy(fontWeight = FontWeight.Bold), color = MaterialTheme.colorScheme.onSurface)
                }
                Text(subtitle, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Spacer(modifier = Modifier.height(8.dp))
            SparkLine(
                data = data,
                modifier = Modifier.fillMaxWidth().height(60.dp),
                lineColor = lineColor,
                fillColor = fillColor
            )
        }
    }
}

@Composable
private fun SparkLine(
    data: List<Float>,
    modifier: Modifier = Modifier,
    lineColor: Color,
    fillColor: Color
) {
    Canvas(modifier = modifier) {
        if (data.size < 2) return@Canvas
        val maxVal = (data.maxOrNull() ?: 100f).coerceAtLeast(1f)
        val w = size.width
        val h = size.height
        val step = w / (data.size - 1)

        val path = Path()
        val fillPath = Path()

        data.forEachIndexed { i, v ->
            val x = i * step
            val y = h - (v / maxVal) * h
            if (i == 0) {
                path.moveTo(x, y)
                fillPath.moveTo(x, h)
                fillPath.lineTo(x, y)
            } else {
                path.lineTo(x, y)
                fillPath.lineTo(x, y)
            }
        }

        fillPath.lineTo(w, h)
        fillPath.close()

        drawPath(fillPath, fillColor)
        drawPath(path, lineColor, style = Stroke(width = 2.dp.toPx()))
    }
}

@Composable
private fun CpuCoresSection(state: DashboardUiState) {
    Text("CPU Cores", style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold), color = MaterialTheme.colorScheme.onSurface)

    val indices = state.coreHistories.keys.sorted()
    for (i in indices.indices step 2) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            val idx0 = indices[i]
            CoreMiniCard(
                modifier = Modifier.weight(1f),
                name = "cpu$idx0",
                usage = state.coreUsages[idx0] ?: 0,
                speedMhz = state.coreSpeeds[idx0] ?: 0,
                history = state.coreHistories[idx0] ?: emptyList()
            )
            if (i + 1 < indices.size) {
                val idx1 = indices[i + 1]
                CoreMiniCard(
                    modifier = Modifier.weight(1f),
                    name = "cpu$idx1",
                    usage = state.coreUsages[idx1] ?: 0,
                    speedMhz = state.coreSpeeds[idx1] ?: 0,
                    history = state.coreHistories[idx1] ?: emptyList()
                )
            } else {
                Spacer(modifier = Modifier.weight(1f))
            }
        }
    }
}

@Composable
private fun CoreMiniCard(
    modifier: Modifier = Modifier,
    name: String,
    usage: Int,
    speedMhz: Int,
    history: List<Float>
) {
    Card(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh)
    ) {
        Column(modifier = Modifier.padding(10.dp)) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text(name.uppercase(), style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 10.sp)
                Text(if (speedMhz > 0) "${speedMhz}MHz" else "~", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 10.sp)
            }
            Text("${usage}%", style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold), color = MaterialTheme.colorScheme.primary)
            Spacer(modifier = Modifier.height(4.dp))
            SparkLine(
                data = history,
                modifier = Modifier.fillMaxWidth().height(30.dp),
                lineColor = Color(0xFF4ADE80),
                fillColor = Color(0xFF4ADE80).copy(alpha = 0.12f)
            )
        }
    }
}
