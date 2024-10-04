#[cfg(test)]
mod tests {
    use crate::color::Color;

    #[test]
    fn test_rgb_to_hsi_red() {
        // Arrange: Set up the test scenario
        let red = 255;
        let green = 0;
        let blue = 0;

        let expected_output = (0_u16, 100_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_green() {
        // Arrange: Set up the test scenario
        let red = 0;
        let green = 255;
        let blue = 0;

        let expected_output = (120_u16, 100_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_blue() {
        // Arrange: Set up the test scenario
        let red = 0;
        let green = 0;
        let blue = 255;

        let expected_output = (240_u16, 100_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_magenta() {
        // Arrange: Set up the test scenario
        let red = 255;
        let green = 0;
        let blue = 255;

        let expected_output = (300_u16, 100_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_cyan() {
        // Arrange: Set up the test scenario
        let red = 0;
        let green = 255;
        let blue = 255;

        let expected_output = (180_u16, 100_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_yellow() {
        // Arrange: Set up the test scenario
        let red = 255;
        let green = 255;
        let blue = 0;

        let expected_output = (60_u16, 100_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_black() {
        // Arrange: Set up the test scenario
        let red = 0;
        let green = 0;
        let blue = 0;

        let expected_output = (0_u16, 0_u8, 0_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_rgb_to_hsi_white() {
        // Arrange: Set up the test scenario
        let red = 255;
        let green = 255;
        let blue = 255;

        let expected_output = (0_u16, 0_u8, 100_u8);

        // Act: Call the function under test
        let result = Color::new(red, green, blue).to_hsv();

        // Assert: Verify the expected output
        assert_eq!(result, expected_output);
    }
}
