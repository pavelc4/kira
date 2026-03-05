package id.xmsaether.kira

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.Build
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import id.xmsaether.kira.core.shizuku.ShizukuHelper
import id.xmsaether.kira.ui.screens.dashboard.DashboardScreen
import id.xmsaether.kira.ui.screens.processes.ProcessListScreen
import id.xmsaether.kira.ui.screens.terminal.TerminalScreen
import id.xmsaether.kira.ui.theme.KiraTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        ShizukuHelper.init()
        enableEdgeToEdge()
        setContent {
            KiraTheme {
                val navController = rememberNavController()
                val items = listOf("Dashboard", "Processes", "Terminal")
                val icons = listOf(Icons.Filled.Home, Icons.Filled.List, Icons.Filled.Build)

                Scaffold(
                    modifier = Modifier.fillMaxSize(),
                    bottomBar = {
                        NavigationBar {
                            val navBackStackEntry by navController.currentBackStackEntryAsState()
                            val currentDestination = navBackStackEntry?.destination
                            items.forEachIndexed { index, screen ->
                                NavigationBarItem(
                                    icon = { Icon(icons[index], contentDescription = screen) },
                                    label = { Text(screen) },
                                    selected = currentDestination?.hierarchy?.any { it.route == screen } == true,
                                    onClick = {
                                        navController.navigate(screen) {
                                            popUpTo(navController.graph.findStartDestination().id) {
                                                saveState = true
                                            }
                                            launchSingleTop = true
                                            restoreState = true
                                        }
                                    }
                                )
                            }
                        }
                    }
                ) { innerPadding ->
                    NavHost(
                        navController = navController,
                        startDestination = "Dashboard",
                        modifier = Modifier.padding(innerPadding)
                    ) {
                        composable("Dashboard") { DashboardScreen() }
                        composable("Processes") { ProcessListScreen() }
                        composable("Terminal") { TerminalScreen() }
                    }
                }
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        ShizukuHelper.destroy()
    }
}