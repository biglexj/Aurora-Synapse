package com.biglexj.aurorasynapse

import android.app.Activity
import android.content.Intent
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class LaunchAppArgs {
    var package_name: String? = null
    var uri: String? = null
    var content: String? = null
}

@TauriPlugin
class LauncherPlugin(private val activity: Activity) : Plugin(activity) {

    @Command
    fun launchApp(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(LaunchAppArgs::class.java)
            val pkg = args.package_name?.trim()?.takeIf { it.isNotEmpty() }
            val content = args.content?.trim()?.takeIf { it.isNotEmpty() }
            val uriStr = args.uri?.trim()?.takeIf { it.isNotEmpty() }

            // 1. Si hay contenido (enlace o texto) y paquete destino (ej. Luna Fetch), intentar ACTION_SEND
            if (pkg != null && content != null) {
                try {
                    val sendIntent = Intent(Intent.ACTION_SEND).apply {
                        type = "text/plain"
                        putExtra(Intent.EXTRA_TEXT, content)
                        setPackage(pkg)
                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    }
                    if (sendIntent.resolveActivity(activity.packageManager) != null) {
                        activity.startActivity(sendIntent)
                        invoke.resolve()
                        return
                    }
                } catch (_: Exception) {
                    // Continuar al siguiente método si falla el intent explícito
                }
            }

            // 2. Intentar apertura mediante esquema URI si se especificó (luna://, supergallery://, elytesia://)
            if (uriStr != null) {
                try {
                    val viewIntent = Intent(Intent.ACTION_VIEW, Uri.parse(uriStr)).apply {
                        if (pkg != null) {
                            setPackage(pkg)
                        }
                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    }
                    if (viewIntent.resolveActivity(activity.packageManager) != null) {
                        activity.startActivity(viewIntent)
                        invoke.resolve()
                        return
                    }
                } catch (_: Exception) {
                    // Continuar a launch intent
                }
            }

            // 3. Apertura directa de la aplicación por paquete (MainActivity)
            if (pkg != null) {
                val launchIntent = activity.packageManager.getLaunchIntentForPackage(pkg)?.apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                if (launchIntent != null) {
                    activity.startActivity(launchIntent)
                    invoke.resolve()
                    return
                }
            }

            // 4. Fallback genérico para esquemas URI
            if (uriStr != null) {
                val fallbackIntent = Intent(Intent.ACTION_VIEW, Uri.parse(uriStr)).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                activity.startActivity(fallbackIntent)
                invoke.resolve()
                return
            }

            invoke.reject("No se encontró la aplicación instalada en este dispositivo.")
        } catch (e: Exception) {
            invoke.reject(e.message ?: "Error al lanzar la aplicación")
        }
    }
}
