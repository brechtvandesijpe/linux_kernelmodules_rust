int rust_helper_led_classdev_register(struct device *parent,
					struct led_classdev *led_cdev)
{
	return led_classdev_register(parent, led_cdev);
}

EXPORT_SYMBOL_GPL(rust_helper_led_classdev_register);