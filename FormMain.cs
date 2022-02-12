using System.Configuration;
using System.Diagnostics;

namespace wpass_gui
{
    public partial class FormMain : Form
    {
        KeyboardHook hook = new();
        public FormMain()
        {
            InitializeComponent();
        }

        private void FormMain_DragDrop(object sender, DragEventArgs e)
        {
            var files = (System.Array)e.Data.GetData(DataFormats.FileDrop);
            var args = new List<string>();
            if ((e.KeyState & 32) == 0) // No Alt key
            {
                args.Add("-n"); // New folder
            }
            if ((e.KeyState & 4) == 4) // Shift key
            {
                args.Add("-D"); // Delete archive file
            }
            args.Add("-l");
            var num = files.Length;
            var finished = 0;
            foreach (var path in files)
            {
                labelProgress.Text = "解压中："+finished+"/"+num;
                var instance = new List<string>(args);
                instance.Add(path.ToString());
                TestCallWPass(instance);
                //CallWPass(instance);
                finished++;
            }
            labelProgress.Text = "解压完成！";
        }

        private void FormMain_DragEnter(object sender, DragEventArgs e)
        {
            if (e.Data.GetDataPresent(DataFormats.FileDrop))
                e.Effect = DragDropEffects.All;                                                              //重要代码：表明是所有类型的数据，比如文件路径
            else
                e.Effect = DragDropEffects.None;

        }

        int CallWPass(List<String> args)
        {
            var settings =
            ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None).AppSettings.Settings;
            if (settings[Constant.WPassPath] == null)
            {
                return -2;
            }
            var path = settings[Constant.WPassPath].Value;
            Process wpass = new Process();
            wpass.StartInfo.CreateNoWindow = true;
            wpass.StartInfo.UseShellExecute = false;
            // wpass.StartInfo.WindowStyle = ProcessWindowStyle.Hidden;
            wpass.StartInfo.FileName = path;
            wpass.StartInfo.Arguments = string.Join(" ", args);
            wpass.StartInfo.RedirectStandardError = true;
            wpass.Start();
            wpass.WaitForExit();
            if (wpass.ExitCode != 0)
            {
                MessageBox.Show(wpass.StandardError.ReadToEnd());
            }
            return wpass.ExitCode;
        }

        int TestCallWPass(List<String> args)
        {
            var settings =
            ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None).AppSettings.Settings;
            if (settings[Constant.WPassPath] == null)
            {
                return -2;
            }
            var path = settings[Constant.WPassPath].Value;
            MessageBox.Show(path + " " + String.Join(" ", args));
            return 0;
        }

        private void buttonSetting_Click(object sender, EventArgs e)
        {
            var formSetting = new FormSetting();
            formSetting.Show();
        }

        private void notifyIcon_MouseClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Right)
            {
                contextMenuStripNotify.Show(Cursor.Position);
            }

            if (e.Button == MouseButtons.Left)
            {
                ToggleForm();
            }
        }

        private void notifyIcon_MouseDoubleClick(object sender, MouseEventArgs e)
        {

        }

        private void FormMain_FormClosing(object sender, FormClosingEventArgs e)
        {
            if (e.CloseReason == CloseReason.UserClosing)
            {
                e.Cancel = true;
                HideForm();
            }
        }

        private void FormMain_Load(object sender, EventArgs e)
        {
            hook.KeyPressed +=
            new EventHandler<KeyPressedEventArgs>(HookKeyPressed);
            hook.RegisterHotKey(wpass_gui.ModifierKeys.Control | wpass_gui.ModifierKeys.Alt,
            Keys.Q);
            this.notifyIcon.Icon = this.Icon;
        }

        void HideForm()
        {
            this.ShowInTaskbar = false;
            this.Visible = false;
        }

        void ShowForm()
        {
            this.WindowState = FormWindowState.Normal;
            this.ShowInTaskbar = true;
            this.Visible = true;
        }

        void ToggleForm()
        {
            if (this.ShowInTaskbar)
            {
                HideForm();
            }
            else
            {
                ShowForm();
            }
        }

        void HookKeyPressed(object sender, KeyPressedEventArgs e)
        {
            ToggleForm();
        }

        private void contextMenuStripNotify_Opening(object sender, System.ComponentModel.CancelEventArgs e)
        {

        }

        private void ToolStripMenuItemExit_Click(object sender, EventArgs e)
        {
            Application.Exit();
        }
    }
}