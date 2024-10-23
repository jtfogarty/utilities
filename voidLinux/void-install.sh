#!/bin/bash
set -e

# Configuration variables
MAIN_DISK="/dev/nvme0n1"  # 476.94 GB NVMe drive
STORAGE_DISK="/dev/sda"    # 931.51 GB SSD
HOSTNAME="void01"
USERNAME="jtfogar"
PASSWORD="yourpassword"
TIMEZONE="America/Chicago"
KEYMAP="us"
LOCALE="en_US.UTF-8"
REPO="https://repo-default.voidlinux.org/current"

# Install required tools
echo "Installing required tools..."
xbps-install -Sy -y \
    parted \
    util-linux \
    gptfdisk \
    dosfstools \
    e2fsprogs \
    xfsprogs \
    lvm2 \
    cryptsetup \
    hdparm \
    psmisc \
    lsof

# Function to completely wipe a disk
wipe_disk() {
    local disk=$1
    echo "Wiping disk $disk..."
    
    # Unmount all partitions
    if [[ $disk == *"nvme"* ]]; then
        for part in ${disk}p*; do
            if [ -e "$part" ]; then
                umount -f $part 2>/dev/null || true
                swapoff $part 2>/dev/null || true
            fi
        done
    else
        for part in ${disk}*; do
            if [ -e "$part" ]; then
                umount -f $part 2>/dev/null || true
                swapoff $part 2>/dev/null || true
            fi
        done
    fi
    
    # Force kernel to reload partition table
    partprobe $disk || true
    
    # Now try to wipe
    wipefs -af $disk
    dd if=/dev/zero of=$disk bs=512 count=1
    sync
    partprobe $disk || true
    
    echo "Finished wiping $disk"
}

echo "Starting installation process..."

# Check if running from live USB
if [ ! -d "/run/rootfsbase" ]; then
    echo "Error: This script must be run from the Void Linux live USB!"
    exit 1
fi

# Wipe disks
echo "Wiping disks..."
wipe_disk "$MAIN_DISK"
wipe_disk "$STORAGE_DISK"

echo "Creating partitions on main disk..."
# Create partition layout on main disk (NVMe)
parted -s $MAIN_DISK -- mklabel gpt
parted -s $MAIN_DISK -- mkpart ESP fat32 1MiB 513MiB
parted -s $MAIN_DISK -- set 1 boot on
parted -s $MAIN_DISK -- mkpart primary linux-swap 513MiB 4609MiB
parted -s $MAIN_DISK -- mkpart primary ext4 4609MiB 100%

echo "Creating partitions on storage disk..."
# Create storage partition
parted -s $STORAGE_DISK -- mklabel gpt
parted -s $STORAGE_DISK -- mkpart primary ext4 1MiB 100%

# Wait for partition table to be reread
sleep 2
partprobe $MAIN_DISK
partprobe $STORAGE_DISK
sleep 2

echo "Formatting partitions..."
# Format main disk partitions (NVMe)
mkfs.fat -F32 "${MAIN_DISK}p1"
mkswap "${MAIN_DISK}p2"
mkfs.ext4 "${MAIN_DISK}p3"

# Format storage partition
mkfs.ext4 "${STORAGE_DISK}1"

echo "Mounting filesystems..."
# Mount the root filesystem
mount "${MAIN_DISK}p3" /mnt

# Create and mount other directories
mkdir -p /mnt/boot/efi
mkdir -p /mnt/storage/hdd

# Mount EFI and storage
mount "${MAIN_DISK}p1" /mnt/boot/efi
mount "${STORAGE_DISK}1" /mnt/storage/hdd
swapon "${MAIN_DISK}p2"

# Mount virtual filesystems needed for chroot
mount --bind /dev /mnt/dev
mount --bind /dev/pts /mnt/dev/pts
mount --bind /proc /mnt/proc
mount --bind /sys /mnt/sys
mount -t efivarfs efivarfs /sys/firmware/efi/efivars || true
mount --bind /sys/firmware/efi/efivars /mnt/sys/firmware/efi/efivars || true

echo "Installing base system..."
# Install base system and desktop environment
XBPS_ARCH="x86_64" xbps-install -Sy -R "$REPO" -r /mnt \
    base-system \
    grub-x86_64-efi \
    efibootmgr \
    NetworkManager \
    vim \
    chrony \
    sudo \
    xorg \
    xfce4 \
    xfce4-terminal \
    xfce4-whiskermenu-plugin \
    xfce4-pulseaudio-plugin \
    network-manager-applet \
    firefox \
    lightdm \
    lightdm-gtk3-greeter \
    pulseaudio \
    pavucontrol \
    arc-theme \
    papirus-icon-theme \
    neofetch

echo "Configuring system..."
# Generate fstab using UUIDs
BOOT_UUID=$(blkid -s UUID -o value ${MAIN_DISK}p1)
SWAP_UUID=$(blkid -s UUID -o value ${MAIN_DISK}p2)
ROOT_UUID=$(blkid -s UUID -o value ${MAIN_DISK}p3)
STORAGE_UUID=$(blkid -s UUID -o value ${STORAGE_DISK}1)

cat > /mnt/etc/fstab <<EOF
# <file system> <dir> <type> <options> <dump> <pass>
UUID=$ROOT_UUID / ext4 defaults 0 1
UUID=$BOOT_UUID /boot/efi vfat defaults 0 2
UUID=$SWAP_UUID none swap defaults 0 0
UUID=$STORAGE_UUID /storage/hdd ext4 defaults 0 2
EOF

# Configure hostname
echo "$HOSTNAME" > /mnt/etc/hostname

# Configure timezone
ln -sf /usr/share/zoneinfo/$TIMEZONE /mnt/etc/localtime

# Configure locale
echo "$LOCALE UTF-8" > /mnt/etc/default/libc-locales
echo "LANG=$LOCALE" > /mnt/etc/locale.conf
echo "KEYMAP=$KEYMAP" > /mnt/etc/vconsole.conf

# Create chroot script
cat > /mnt/tmp/chroot-setup.sh <<EOF
#!/bin/bash

# Set root password
echo "root:$PASSWORD" | chpasswd

# Create user
useradd -m -G wheel,users,audio,video,cdrom,input -s /bin/bash "$USERNAME"
echo "$USERNAME:$PASSWORD" | chpasswd

# Configure sudo
echo "%wheel ALL=(ALL) ALL" > /etc/sudoers.d/wheel

# Configure storage permissions
mkdir -p /storage/hdd
chown $USERNAME:$USERNAME /storage/hdd
chmod 755 /storage/hdd

# Install and configure GRUB
mkdir -p /boot/grub
grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=BOOT --removable
grub-mkconfig -o /boot/grub/grub.cfg

# Enable services
ln -s /etc/sv/NetworkManager /etc/runit/runsvdir/default/
ln -s /etc/sv/dbus /etc/runit/runsvdir/default/
ln -s /etc/sv/chronyd /etc/runit/runsvdir/default/
ln -s /etc/sv/lightdm /etc/runit/runsvdir/default/

# Generate locales
xbps-reconfigure -f glibc-locales

# Configure LightDM for automatic login (optional)
cat > /etc/lightdm/lightdm.conf <<LIGHTDMEOF
[Seat:*]
autologin-user=$USERNAME
autologin-session=xfce
LIGHTDMEOF

# Set default XFCE theme
mkdir -p /home/$USERNAME/.config/xfce4/xfconf/xfce-perchannel-xml/
cat > /home/$USERNAME/.config/xfce4/xfconf/xfce-perchannel-xml/xsettings.xml <<THEMEEOF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xsettings" version="1.0">
  <property name="Net" type="empty">
    <property name="ThemeName" type="string" value="Arc-Dark"/>
    <property name="IconThemeName" type="string" value="Papirus-Dark"/>
  </property>
</channel>
THEMEEOF
chown -R $USERNAME:$USERNAME /home/$USERNAME/.config
EOF

# Make the script executable and run it in chroot
chmod +x /mnt/tmp/chroot-setup.sh
chroot /mnt /tmp/chroot-setup.sh

# Clean up
rm -f /mnt/tmp/chroot-setup.sh

echo "Unmounting filesystems..."
# Unmount everything in reverse order
umount /mnt/sys/firmware/efi/efivars || true
umount /mnt/sys
umount /mnt/proc
umount /mnt/dev/pts
umount /mnt/dev
umount /mnt/storage/hdd
umount /mnt/boot/efi
swapoff "${MAIN_DISK}p2"
umount /mnt

echo "Installation complete! You can now reboot."
