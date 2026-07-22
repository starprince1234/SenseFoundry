package com.sensefoundry.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private val Pine = Color(0xFF174D3C)
private val PineDeep = Color(0xFF0B3026)
private val Jade = Color(0xFF4E8A72)
private val Cinnabar = Color(0xFFB54A34)
private val Paper = Color(0xFFF7F3EA)
private val PaperDeep = Color(0xFFEDE5D6)
private val Ink = Color(0xFF17211D)
private val Mist = Color(0xFFDCE8E1)

private val LightColors = lightColorScheme(
    primary = Pine,
    onPrimary = Color.White,
    primaryContainer = Mist,
    onPrimaryContainer = PineDeep,
    secondary = Cinnabar,
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFFFDBD2),
    onSecondaryContainer = Color(0xFF3D0700),
    tertiary = Jade,
    background = Paper,
    onBackground = Ink,
    surface = Color(0xFFFFFCF6),
    onSurface = Ink,
    surfaceVariant = PaperDeep,
    onSurfaceVariant = Color(0xFF4D5B55),
    outline = Color(0xFF78847E),
)

private val DarkColors = darkColorScheme(
    primary = Color(0xFF9BD5BC),
    onPrimary = Color(0xFF003827),
    primaryContainer = Color(0xFF16513D),
    onPrimaryContainer = Color(0xFFB7F1D5),
    secondary = Color(0xFFFFB4A3),
    onSecondary = Color(0xFF661407),
    secondaryContainer = Color(0xFF8C2D1D),
    onSecondaryContainer = Color(0xFFFFDAD2),
    background = Color(0xFF101713),
    onBackground = Color(0xFFE1EAE4),
    surface = Color(0xFF151D19),
    onSurface = Color(0xFFE1EAE4),
    surfaceVariant = Color(0xFF24332C),
    onSurfaceVariant = Color(0xFFBCCAC2),
)

private val AppTypography = Typography(
    displaySmall = TextStyle(
        fontFamily = FontFamily.Serif,
        fontWeight = FontWeight.Bold,
        fontSize = 40.sp,
        lineHeight = 48.sp,
    ),
    headlineLarge = TextStyle(
        fontFamily = FontFamily.Serif,
        fontWeight = FontWeight.Bold,
        fontSize = 30.sp,
        lineHeight = 38.sp,
    ),
    headlineMedium = TextStyle(
        fontFamily = FontFamily.Serif,
        fontWeight = FontWeight.SemiBold,
        fontSize = 24.sp,
        lineHeight = 32.sp,
    ),
    titleLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Bold,
        fontSize = 20.sp,
        lineHeight = 28.sp,
    ),
    titleMedium = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.SemiBold,
        fontSize = 16.sp,
        lineHeight = 24.sp,
    ),
    bodyLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontSize = 16.sp,
        lineHeight = 26.sp,
    ),
    bodyMedium = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontSize = 14.sp,
        lineHeight = 22.sp,
    ),
    labelLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.SemiBold,
        fontSize = 14.sp,
    ),
)

private val AppShapes = Shapes(
    small = RoundedCornerShape(10.dp),
    medium = RoundedCornerShape(18.dp),
    large = RoundedCornerShape(28.dp),
)

@Composable
fun SenseFoundryTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) DarkColors else LightColors,
        typography = AppTypography,
        shapes = AppShapes,
        content = content,
    )
}
