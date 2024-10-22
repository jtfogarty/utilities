#!/bin/bash
set -e

# Configuration variables
MAIN_DISK="/dev/sdb"  # 58.6G - Will be used for OS installation
STORAGE_DISK="/dev/sda"  # 931.5G - Will be used for storage
NVME_DISK="/dev/nvme0n1"  # 476.9G - Will be used for storage
HOSTNAME="void01"
USERNAME="jtfogar"
PASSWORD="yourpassword"
TIMEZONE="America/Chicago"
KEYMAP="us"

# Function to completely wipe a disk
wipe_disk() {
    local disk=$1
    echo "Wiping disk $disk..."
    # Delete LVM if exists
    if vgdisplay ubuntu-vg >/dev/null 2>&1; then
        vgremove -f ubuntu-vg
    fi
    # Delete all partitions
    wipefs -a $disk
    dd if=/dev/zero of=$disk bs=512 count=1
    sync
}

# Wipe all disks
wipe_disk $MAIN_DISK
wipe_disk $STORAGE_DISK
wipe_disk $NVME_DISK

# Create partition layout
parted -s $MAIN_DISK mklabel gpt
parted -s $MAIN_DISK mkpart ESP fat32 1MiB 513MiB
parted -s $MAIN_DISK set 1 boot on
parted -s $MAIN_DISK mkpart primary linux-swap 513MiB 4609MiB
parted -s $MAIN_DISK mkpart primary ext4 4609MiB 100%

# Format main disk partitions
mkfs.fat -F32 ${MAIN_DISK}1
mkswap ${MAIN_DISK}2
mkfs.ext4 ${MAIN_DISK}3

# Create storage partitions
parted -s $STORAGE_DISK mklabel gpt
parted -s $STORAGE_DISK mkpart primary ext4 1MiB 100%
mkfs.ext4 ${STORAGE_DISK}1

parted -s $NVME_DISK mklabel gpt
parted -s $NVME_DISK mkpart primary ext4 1MiB 100%
mkfs.ext4 ${NVME_DISK}1

# Create void-installer configuration
cat > /tmp/void-installer.conf <<EOF
# Void Linux installer configuration
HOSTNAME="$HOSTNAME"
KEYBOARD="$KEYMAP"
TIMEZONE="$TIMEZONE"
HARDWARECLOCK="UTC"
KEYMAP="$KEYMAP"
FONT="lat9w-16"
TTYS=2
ROOT_PASSWORD="$PASSWORD"
USER_NAME="$USERNAME"
USER_PASSWORD="$PASSWORD"
USER_GROUPS="wheel,users,audio,video,cdrom,input"
USER_SHELL="/bin/bash"
BOOTLOADER="grub"
BOOTLOADER_SET="yes"
ROOTFS="ext4"
ROOT_DEVICE="${MAIN_DISK}3"
ESP_DEVICE="${MAIN_DISK}1"
SWAP_DEVICE="${MAIN_DISK}2"
EOF

# Run void-installer with configuration
void-installer -a /tmp/void-installer.conf

# After installation, mount the system to add storage drives
mount ${MAIN_DISK}3 /mnt

# Create storage mount points
mkdir -p /mnt/storage/hdd
mkdir -p /mnt/storage/nvme

# Add storage drives to fstab
cat >> /mnt/etc/fstab <<EOF

# Storage drives
${STORAGE_DISK}1  /storage/hdd   ext4  defaults  0  2
${NVME_DISK}1     /storage/nvme  ext4  defaults  0  2
EOF

# Chroot to configure storage permissions
xchroot /mnt /bin/bash <<EOF
# Create storage directories with proper permissions
mkdir -p /storage/hdd
mkdir -p /storage/nvme
chmod 755 /storage/hdd /storage/nvme
chown $USERNAME:$USERNAME /storage/hdd /storage/nvme

# Enable NetworkManager
ln -s /etc/sv/NetworkManager /etc/runit/runsvdir/default/
ln -s /etc/sv/dbus /etc/runit/runsvdir/default/
EOF

# Unmount everything
umount -R /mnt
echo "Installation complete! You can now reboot."
